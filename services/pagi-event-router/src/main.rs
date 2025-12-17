use axum::{
    extract::State,
    http::StatusCode,
    response::{sse::Event, sse::Sse, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use futures_util::StreamExt;
use pagi_common::EventEnvelope;
use rdkafka::{
    admin::{AdminClient, AdminOptions, NewTopic, TopicReplication},
    consumer::{Consumer, StreamConsumer},
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
    Message,
};
use std::{convert::Infallible, net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

const TOPIC: &str = "core-events";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Kafka,
    Sse,
}

#[derive(Clone)]
struct AppState {
    mode: Mode,
    producer: Option<FutureProducer>,
    broadcast_tx: broadcast::Sender<EventEnvelope>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pagi_http::tracing::init("pagi-event-router");

    let mode = match std::env::var("EVENT_ROUTER_MODE")
        .unwrap_or_else(|_| "kafka".to_string())
        .to_lowercase()
        .as_str()
    {
        "sse" => Mode::Sse,
        _ => Mode::Kafka,
    };

    let (broadcast_tx, _rx) = broadcast::channel::<EventEnvelope>(1024);

    let (producer, consumer) = if mode == Mode::Kafka {
        let brokers = std::env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
        tracing::info!(%brokers, "starting in kafka mode");

        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &brokers)
            .set("message.timeout.ms", "5000")
            .create()?;

        // Create topic if needed (best-effort).
        ensure_topic(&brokers).await;

        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &brokers)
            .set("group.id", "pagi-event-router")
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .create()?;
        consumer.subscribe(&[TOPIC])?;

        (Some(producer), Some(consumer))
    } else {
        tracing::info!("starting in sse mode");
        (None, None)
    };

    let state = Arc::new(AppState {
        mode,
        producer,
        broadcast_tx,
    });

    if let Some(consumer) = consumer {
        let state_clone = state.clone();
        tokio::spawn(async move {
            consume_and_broadcast(consumer, state_clone).await;
        });
    }

    let app = Router::new()
        .route("/health", get(health))
        .route("/healthz", get(health))
        .route("/publish", post(publish))
        .route("/events", get(events))
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

    match state.mode {
        Mode::Sse => {
            let _ = state.broadcast_tx.send(ev);
            StatusCode::ACCEPTED.into_response()
        }
        Mode::Kafka => {
            let Some(producer) = &state.producer else {
                return (StatusCode::INTERNAL_SERVER_ERROR, "kafka producer missing").into_response();
            };

            let payload = match serde_json::to_string(&ev) {
                Ok(s) => s,
                Err(e) => return (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
            };

            let key = ev
                .twin_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| ev.id.to_string());

            let record = FutureRecord::to(TOPIC).payload(&payload).key(&key);
            match producer.send(record, Duration::from_secs(5)).await {
                Ok(_) => StatusCode::ACCEPTED.into_response(),
                Err((e, _)) => (StatusCode::BAD_GATEWAY, e.to_string()).into_response(),
            }
        }
    }
}

async fn events(State(state): State<Arc<AppState>>) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.broadcast_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|msg| async move {
        match msg {
            Ok(ev) => {
                let data = match serde_json::to_string(&ev) {
                    Ok(s) => s,
                    Err(_) => return None,
                };
                Some(Ok(Event::default().event(ev.event_type).data(data)))
            }
            Err(_) => None,
        }
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

async fn consume_and_broadcast(consumer: StreamConsumer, state: Arc<AppState>) {
    let mut stream = consumer.stream();
    while let Some(msg) = stream.next().await {
        match msg {
            Ok(m) => {
                if let Some(payload) = m.payload() {
                    match serde_json::from_slice::<EventEnvelope>(payload) {
                        Ok(envelope) => {
                            tracing::info!(event_type = %envelope.event_type, "consumed");
                            let _ = state.broadcast_tx.send(envelope);
                        }
                        Err(err) => {
                            tracing::warn!(error = %err, "failed to decode event envelope");
                        }
                    }
                }
            }
            Err(err) => {
                tracing::error!(error = %err, "kafka consumer error");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
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
