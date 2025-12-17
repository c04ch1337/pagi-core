use clap::Parser;
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use reqwest::Client;

#[derive(Debug, Parser)]
struct Args {
    /// Event router base URL (expects /events SSE endpoint)
    #[arg(long, env = "EVENT_ROUTER_URL", default_value = "http://127.0.0.1:7001")]
    event_router_url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();
    let url = format!("{}/events", args.event_router_url.trim_end_matches('/'));
    tracing::info!(%url, "subscribing");

    let client = Client::new();
    let resp = client.get(url).send().await?;

    let mut stream = resp.bytes_stream().eventsource();
    while let Some(evt) = stream.next().await {
        match evt {
            Ok(ev) => {
                // `ev.data` is the JSON string published by PAGI-EventRouter.
                tracing::info!(event = %ev.data, "event");
            }
            Err(err) => {
                tracing::warn!(error = %err, "event stream error");
            }
        }
    }

    Ok(())
}
