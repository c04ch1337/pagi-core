use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use pagi_common::EventEnvelope;
use rdkafka::{
    admin::{AdminClient, AdminOptions, NewTopic, TopicReplication},
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

const TOPIC: &str = "core-events";

#[derive(Clone)]
struct AppState {
    producer: FutureProducer,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pagi_http::tracing::init("pagi-event-router");

    let brokers = std::env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
    tracing::info!(%brokers, "starting in kafka mode (kafka-only)");

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &brokers)
        .set("message.timeout.ms", "5000")
        .create()?;

    // Create topic if needed (best-effort).
    ensure_topic(&brokers).await;

    let state = Arc::new(AppState {
        producer,
    });

    let app = Router::new()
        .route("/healthz", get(health))
        .route("/publish", post(publish))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let addr: SocketAddr = pagi_http::config::bind_addr(([0, 0, 0, 0], 8000).into());
    tracing::info!(%addr, "listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

async fn publish(State(state): State<Arc<AppState>>, Json(mut ev): Json<EventEnvelope>) -> impl IntoResponse {
    if ev.event_type.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "event_type required").into_response();
    }
    if ev.source.is_none() {
        ev.source = Some("pagi-event-router".to_string());
    }

    let payload = match serde_json::to_string(&ev) {
        Ok(s) => s,
        Err(e) => return (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    };

    let key = ev
        .twin_id
        .map(|id| id.to_string())
        .unwrap_or_else(|| ev.id.to_string());

    let record = FutureRecord::to(TOPIC).payload(&payload).key(&key);
    match state.producer.send(record, Duration::from_secs(5)).await {
        Ok(_) => StatusCode::ACCEPTED.into_response(),
        Err((e, _)) => (StatusCode::BAD_GATEWAY, e.to_string()).into_response(),
    }
}

async fn ensure_topic(brokers: &str) {
    let admin: AdminClient<_> = match ClientConfig::new().set("bootstrap.servers", brokers).create() {
        Ok(a) => a,
        Err(err) => {
            tracing::warn!(error = %err, "failed to create kafka admin client");
            return;
        }
    };

    let new_topic = NewTopic::new(TOPIC, 1, TopicReplication::Fixed(1));
    match admin.create_topics([&new_topic], &AdminOptions::new()).await {
        Ok(results) => {
            for res in results {
                match res {
                    Ok(name) => tracing::info!(%name, "topic ready"),
                    Err((name, err)) => tracing::info!(%name, error = %err, "topic create skipped/failed"),
                }
            }
        }
        Err(err) => {
            tracing::warn!(error = %err, "create_topics failed");
        }
    }
}
