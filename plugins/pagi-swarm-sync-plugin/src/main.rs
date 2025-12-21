use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use git2::{build::CheckoutBuilder, Cred, FetchOptions, PushOptions, RemoteCallbacks, Repository, Signature};
use pagi_common::{PagiError, Playbook, RefinementArtifact, TwinId};
use pagi_http::errors::PagiAxumError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    net::SocketAddr,
    path::PathBuf,
};
use tracing::{error, info};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    http: reqwest::Client,
    external_gateway_url: String,
    plugin_url: String,
    git: GitConfig,
}

#[derive(Clone, Debug)]
struct GitConfig {
    repo_url: String,
    local_path: PathBuf,
    base_branch: String,
    author_name: String,
    author_email: String,
    git_username: Option<String>,
    git_token: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pagi_http::tracing::init("pagi-swarm-sync-plugin");

    let external_gateway_url = std::env::var("EXTERNAL_GATEWAY_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8010".to_string());

    // This is the URL ExternalGateway should use to reach this plugin.
    let plugin_url = std::env::var("PLUGIN_URL").unwrap_or_else(|_| "http://127.0.0.1:9010".to_string());

    let git = GitConfig {
        repo_url: std::env::var("SWARM_REPO_URL").unwrap_or_default(),
        local_path: std::env::var("SWARM_LOCAL_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/data/repo")),
        base_branch: std::env::var("SWARM_BASE_BRANCH").unwrap_or_else(|_| "main".to_string()),
        author_name: std::env::var("GIT_AUTHOR_NAME").unwrap_or_else(|_| "PAGI-MO".to_string()),
        author_email: std::env::var("GIT_AUTHOR_EMAIL").unwrap_or_else(|_| "mo@pagi.local".to_string()),
        git_username: std::env::var("GIT_USERNAME").ok(),
        git_token: std::env::var("GIT_TOKEN").ok(),
    };

    let state = AppState {
        http: reqwest::Client::new(),
        external_gateway_url,
        plugin_url,
        git,
    };

    // Best-effort: register tools with ExternalGateway on startup.
    let state_clone = state.clone();
    tokio::spawn(async move {
        if let Err(err) = register_tools_with_gateway(&state_clone).await {
            error!(error = %err, "failed to register tools with ExternalGateway");
        }
    });

    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/push_artifact", post(push_artifact))
        .route("/pull_latest_playbook", post(pull_latest_playbook))
        .with_state(state)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let addr: SocketAddr = pagi_http::config::bind_addr(([0, 0, 0, 0], 9010).into());
    info!(%addr, "pagi-swarm-sync-plugin listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
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

async fn register_tools_with_gateway(state: &AppState) -> Result<(), String> {
    let gateway = state.external_gateway_url.trim_end_matches('/');
    let register_url = format!("{gateway}/register_tool");

    let tools = vec![
        GatewayToolSchema {
            name: "swarm_sync_push_artifact".to_string(),
            description: "Push a refinement artifact into the swarm sync backend (e.g., GitHub)".to_string(),
            plugin_url: state.plugin_url.clone(),
            endpoint: "/push_artifact".to_string(),
            parameters: json!({
                "type": "object",
                "description": "RefinementArtifact payload",
            }),
        },
        GatewayToolSchema {
            name: "swarm_sync_pull_latest_playbook".to_string(),
            description: "Pull the latest playbook from the swarm sync backend".to_string(),
            plugin_url: state.plugin_url.clone(),
            endpoint: "/pull_latest_playbook".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "twin_id": {"type": "string"}
                }
            }),
        },
    ];

    for tool in tools {
        let payload = GatewayRegisterPayload {
            twin_id: None,
            tool,
        };

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

    info!("registered swarm sync tools with ExternalGateway");
    Ok(())
}

async fn push_artifact(State(state): State<AppState>, Json(artifact): Json<RefinementArtifact>) -> impl IntoResponse {
    let cfg = state.git.clone();

    match tokio::task::spawn_blocking(move || git_push_artifact(&cfg, &artifact))
        .await
        .map_err(|e| e.to_string())
    {
        Ok(Ok(branch)) => (StatusCode::OK, format!("pushed artifact on branch {branch}")).into_response(),
        Ok(Err(err)) => PagiAxumError::with_status(PagiError::plugin_exec(err), StatusCode::BAD_GATEWAY).into_response(),
        Err(join_err) => PagiAxumError::with_status(PagiError::plugin_exec(join_err), StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

#[derive(Debug, Deserialize)]
struct PullLatestRequest {
    #[allow(dead_code)]
    pub twin_id: Option<TwinId>,
}

async fn pull_latest_playbook(
    State(state): State<AppState>,
    Json(_req): Json<PullLatestRequest>,
) -> impl IntoResponse {
    let cfg = state.git.clone();

    match tokio::task::spawn_blocking(move || git_pull_latest_playbook(&cfg))
        .await
        .map_err(|e| e.to_string())
    {
        Ok(Ok(playbook)) => (StatusCode::OK, Json(playbook)).into_response(),
        Ok(Err(err)) => PagiAxumError::with_status(PagiError::plugin_exec(err), StatusCode::BAD_GATEWAY).into_response(),
        Err(join_err) => PagiAxumError::with_status(PagiError::plugin_exec(join_err), StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

fn git_callbacks(cfg: &GitConfig) -> RemoteCallbacks<'static> {
    let username = cfg.git_username.clone().unwrap_or_else(|| "git".to_string());
    let token = cfg.git_token.clone();

    let mut cb = RemoteCallbacks::new();
    cb.credentials(move |_url, _username_from_url, _allowed| {
        if let Some(tok) = token.as_deref() {
            Cred::userpass_plaintext(&username, tok)
        } else {
            Cred::default()
        }
    });
    cb
}

fn open_or_clone(cfg: &GitConfig) -> Result<Repository, String> {
    if cfg.local_path.join(".git").exists() {
        return Repository::open(&cfg.local_path).map_err(|e| e.to_string());
    }
    if cfg.repo_url.trim().is_empty() {
        return Err("SWARM_REPO_URL is required (no existing repo at SWARM_LOCAL_PATH)".to_string());
    }
    std::fs::create_dir_all(&cfg.local_path).map_err(|e| e.to_string())?;
    Repository::clone(&cfg.repo_url, &cfg.local_path).map_err(|e| e.to_string())
}

fn git_push_artifact(cfg: &GitConfig, artifact: &RefinementArtifact) -> Result<String, String> {
    let repo = open_or_clone(cfg)?;

    // Create a unique improvement branch.
    let branch = format!(
        "improvement/{}-{}",
        time::OffsetDateTime::now_utc().unix_timestamp(),
        Uuid::new_v4()
    );

    // Base off current HEAD.
    let head_commit = repo
        .head()
        .map_err(|e| e.to_string())?
        .peel_to_commit()
        .map_err(|e| e.to_string())?;

    repo.branch(&branch, &head_commit, true)
        .map_err(|e| e.to_string())?;
    repo.set_head(&format!("refs/heads/{branch}"))
        .map_err(|e| e.to_string())?;
    repo.checkout_head(Some(CheckoutBuilder::new().force()))
        .map_err(|e| e.to_string())?;

    // Write artifact to a deterministic location.
    let artifacts_dir = cfg.local_path.join("swarm_artifacts");
    std::fs::create_dir_all(&artifacts_dir).map_err(|e| e.to_string())?;

    let artifact_id = Uuid::new_v4();
    let rel_path = PathBuf::from(format!("swarm_artifacts/{artifact_id}.toml"));
    let full_path = cfg.local_path.join(&rel_path);
    let artifact_toml = toml::to_string_pretty(artifact).map_err(|e| e.to_string())?;
    std::fs::write(&full_path, artifact_toml).map_err(|e| e.to_string())?;

    // Stage + commit.
    let mut index = repo.index().map_err(|e| e.to_string())?;
    index.add_path(&rel_path).map_err(|e| e.to_string())?;
    index.write().map_err(|e| e.to_string())?;

    let tree_id = index.write_tree().map_err(|e| e.to_string())?;
    let tree = repo.find_tree(tree_id).map_err(|e| e.to_string())?;

    let sig = Signature::now(&cfg.author_name, &cfg.author_email).map_err(|e| e.to_string())?;
    let parent = repo
        .head()
        .map_err(|e| e.to_string())?
        .peel_to_commit()
        .map_err(|e| e.to_string())?;

    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        "Add refinement artifact",
        &tree,
        &[&parent],
    )
    .map_err(|e| e.to_string())?;

    // Push branch.
    let mut remote = repo.find_remote("origin").map_err(|e| e.to_string())?;
    let mut push_opts = PushOptions::new();
    push_opts.remote_callbacks(git_callbacks(cfg));

    let refspec = format!("refs/heads/{branch}:refs/heads/{branch}");
    remote
        .push(&[&refspec], Some(&mut push_opts))
        .map_err(|e| e.to_string())?;

    Ok(branch)
}

fn git_pull_latest_playbook(cfg: &GitConfig) -> Result<Playbook, String> {
    let repo = open_or_clone(cfg)?;

    // Best-effort fetch base branch.
    let mut remote = repo.find_remote("origin").map_err(|e| e.to_string())?;
    let mut fo = FetchOptions::new();
    fo.remote_callbacks(git_callbacks(cfg));
    remote
        .fetch(&[cfg.base_branch.as_str()], Some(&mut fo), None)
        .map_err(|e| e.to_string())?;

    // Read playbook from working tree.
    let playbook_path = cfg.local_path.join("playbook.toml");
    if !playbook_path.exists() {
        return Ok(Playbook::default());
    }
    let playbook_str = std::fs::read_to_string(&playbook_path).map_err(|e| e.to_string())?;
    toml::from_str::<Playbook>(&playbook_str).map_err(|e| e.to_string())
}
