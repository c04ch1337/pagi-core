use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use pagi_common::TwinId;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    net::SocketAddr,
    sync::Arc,
};
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    http: reqwest::Client,
    external_gateway_url: String,
    plugin_url: String,

    /// Canonical actor id (URL). Example: https://pagi.example.com/actor/mo
    actor_id: String,
    /// Public key PEM published in the actor object (optional).
    public_key_pem: Option<String>,

    /// If set, the plugin will POST Create(Note) activities to this outbox URL.
    /// (Best-effort; many servers require additional auth / signatures.)
    outbox_url: Option<String>,

    /// If true, also attempt delivery to follower inbox URLs.
    deliver_to_followers: bool,

    followers: Arc<RwLock<Vec<String>>>,
    outbox: Arc<RwLock<Vec<serde_json::Value>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GatewayRegisterPayload {
    twin_id: Option<TwinId>,
    tool: GatewayToolSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GatewayToolSchema {
    name: String,
    description: String,
    plugin_url: String,
    endpoint: String,
    parameters: serde_json::Value,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pagi_http::tracing::init("pagi-activitypub-plugin");

    let external_gateway_url = std::env::var("EXTERNAL_GATEWAY_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8010".to_string());
    let plugin_url = std::env::var("PLUGIN_URL").unwrap_or_else(|_| "http://127.0.0.1:9070".to_string());

    let actor_id = std::env::var("ACTIVITYPUB_ACTOR_ID")
        .unwrap_or_else(|_| "https://pagi.example.com/actor/mo".to_string());

    let public_key_pem = std::env::var("ACTIVITYPUB_PUBLIC_KEY_PEM").ok();
    let outbox_url = std::env::var("ACTIVITYPUB_OUTBOX_URL").ok();
    let deliver_to_followers = std::env::var("ACTIVITYPUB_DELIVER_TO_FOLLOWERS")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase()
        == "true";

    let state = AppState {
        http: reqwest::Client::new(),
        external_gateway_url,
        plugin_url,
        actor_id,
        public_key_pem,
        outbox_url,
        deliver_to_followers,
        followers: Arc::new(RwLock::new(Vec::new())),
        outbox: Arc::new(RwLock::new(Vec::new())),
    };

    // Best-effort: register tools with ExternalGateway on startup.
    let st = state.clone();
    tokio::spawn(async move {
        if let Err(err) = register_tools_with_gateway(&st).await {
            error!(error = %err, "failed to register ActivityPub tools with ExternalGateway");
        }
    });

    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/actor", get(get_actor))
        .route("/inbox", post(inbox))
        .route("/outbox", get(get_outbox))
        .route("/followers", get(get_followers_http))
        // ExternalGateway tools:
        .route("/publish_note", post(publish_note))
        .route("/follow_actor", post(follow_actor))
        .route("/get_followers", post(get_followers_tool))
        .with_state(state)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let addr: SocketAddr = pagi_http::config::bind_addr(([0, 0, 0, 0], 9070).into());
    info!(%addr, "pagi-activitypub-plugin listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn register_tools_with_gateway(state: &AppState) -> Result<(), String> {
    let gateway = state.external_gateway_url.trim_end_matches('/');
    let register_url = format!("{gateway}/register_tool");

    let tools = vec![
        GatewayToolSchema {
            name: "publish_note".to_string(),
            description: "Publish a public ActivityPub Note (best-effort outbound posting)".to_string(),
            plugin_url: state.plugin_url.clone(),
            endpoint: "/publish_note".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "content": {"type": "string"},
                    "to": {"type": "array", "items": {"type": "string"}}
                },
                "required": ["content"]
            }),
        },
        GatewayToolSchema {
            name: "follow_actor".to_string(),
            description: "Add an ActivityPub actor URL to the local followers list (MVP)".to_string(),
            plugin_url: state.plugin_url.clone(),
            endpoint: "/follow_actor".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "actor": {"type": "string"}
                },
                "required": ["actor"]
            }),
        },
        GatewayToolSchema {
            name: "get_followers".to_string(),
            description: "Return the local followers list (MVP)".to_string(),
            plugin_url: state.plugin_url.clone(),
            endpoint: "/get_followers".to_string(),
            parameters: json!({"type": "object"}),
        },
    ];

    for tool in tools {
        let payload = GatewayRegisterPayload { twin_id: None, tool };
        state
            .http
            .post(&register_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;
    }

    info!("registered ActivityPub tools with ExternalGateway");
    Ok(())
}

async fn get_actor(State(state): State<AppState>) -> impl IntoResponse {
    let actor_id = state.actor_id.trim_end_matches('/');
    let inbox = format!("{}/inbox", actor_id);
    let outbox = format!("{}/outbox", actor_id);
    let followers = format!("{}/followers", actor_id);

    let mut actor = json!({
        "@context": ["https://www.w3.org/ns/activitystreams"],
        "id": actor_id,
        "type": "Person",
        "preferredUsername": "pagi_mo",
        "name": "PAGI Master Orchestrator",
        "inbox": inbox,
        "outbox": outbox,
        "followers": followers,
    });

    if let Some(pk) = state.public_key_pem.as_deref() {
        actor["publicKey"] = json!({
            "id": format!("{}#main-key", actor_id),
            "owner": actor_id,
            "publicKeyPem": pk,
        });
    }

    (StatusCode::OK, Json(actor)).into_response()
}

async fn inbox(Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    // MVP: accept inbound but do not process federation yet.
    tracing::debug!("inbox received: {}", payload);
    StatusCode::OK
}

async fn get_outbox(State(state): State<AppState>) -> impl IntoResponse {
    let out = state.outbox.read().await.clone();
    (StatusCode::OK, Json(json!({"orderedItems": out}))).into_response()
}

async fn get_followers_http(State(state): State<AppState>) -> impl IntoResponse {
    let followers = state.followers.read().await.clone();
    (StatusCode::OK, Json(json!({"followers": followers}))).into_response()
}

#[derive(Debug, Deserialize)]
struct FollowActorRequest {
    actor: String,
}

async fn follow_actor(State(state): State<AppState>, Json(req): Json<FollowActorRequest>) -> impl IntoResponse {
    let mut followers = state.followers.write().await;
    if !followers.contains(&req.actor) {
        followers.push(req.actor.clone());
    }
    (StatusCode::OK, Json(json!({"ok": true, "followers": followers.clone()}))).into_response()
}

#[derive(Debug, Deserialize)]
struct PublishNoteRequest {
    content: String,
    #[serde(default)]
    to: Option<Vec<String>>,
}

async fn publish_note(State(state): State<AppState>, Json(req): Json<PublishNoteRequest>) -> impl IntoResponse {
    let now = time::OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339).unwrap_or_default();
    let actor = state.actor_id.trim_end_matches('/').to_string();
    let to = req.to.unwrap_or_else(|| vec!["https://www.w3.org/ns/activitystreams#Public".to_string()]);

    let activity_id = format!("{}/activities/{}", actor, Uuid::new_v4());
    let note_id = format!("{}/notes/{}", actor, Uuid::new_v4());

    let activity = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": activity_id,
        "type": "Create",
        "actor": actor,
        "object": {
            "id": note_id,
            "type": "Note",
            "published": now,
            "attributedTo": state.actor_id,
            "content": req.content,
            "to": to,
        }
    });

    // Store locally.
    state.outbox.write().await.push(activity.clone());

    // Best-effort remote post.
    if let Some(outbox) = state.outbox_url.as_deref() {
        if let Err(err) = state.http.post(outbox).json(&activity).send().await {
            warn!(error = %err, "outbox delivery failed (best-effort)");
        }
    }

    if state.deliver_to_followers {
        let followers = state.followers.read().await.clone();
        for inbox_url in followers {
            if let Err(err) = state.http.post(&inbox_url).json(&activity).send().await {
                warn!(inbox = %inbox_url, error = %err, "follower delivery failed (best-effort)");
            }
        }
    }

    (StatusCode::OK, Json(json!({"ok": true, "activity": activity}))).into_response()
}

async fn get_followers_tool(State(state): State<AppState>, Json(_req): Json<serde_json::Value>) -> impl IntoResponse {
    let followers = state.followers.read().await.clone();
    (StatusCode::OK, Json(json!({"followers": followers}))).into_response()
}

