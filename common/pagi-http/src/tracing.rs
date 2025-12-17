use tracing_subscriber::{fmt, EnvFilter};

pub fn init(service_name: &str) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_level(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_ansi(true)
        .with_span_events(fmt::format::FmtSpan::CLOSE)
        .with_timer(fmt::time::UtcTime::rfc_3339())
        .with_writer(std::io::stderr)
        .init();

    tracing::info!(service = service_name, "tracing initialized");
}
