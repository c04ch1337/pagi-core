use axum::{
    extract::State,
    http::StatusCode,
    response::{sse::Event, sse::Sse, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use futures_util::StreamExt;
use pagi_common::EventEnvelope;
use std::{convert::Infallible, net::SocketAddr, time::Duration};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<EventEnvelope>,
}

#[tokio::main]
async fn main() {
    pagi_http::tracing::init("pagi-event-router");

    let (tx, _rx) = broadcast::channel::<EventEnvelope>(1024);
    let state = AppState { tx };

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/publish", post(publish))
        .route("/events", get(events))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let addr: SocketAddr = pagi_http::config::bind_addr(([0, 0, 0, 0], 7001).into());
    tracing::info!(%addr, "listening");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

async fn publish(State(state): State<AppState>, Json(ev): Json<EventEnvelope>) -> impl IntoResponse {
    // Ensure server-side timestamp exists; if not, set it.
    if ev.event_type.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "event_type required").into_response();
    }
    let _ = state.tx.send(ev);
    StatusCode::ACCEPTED.into_response()
}

async fn events(State(state): State<AppState>) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();
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
