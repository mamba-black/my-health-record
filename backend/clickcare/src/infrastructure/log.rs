use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format::FmtSpan;

pub fn init_logger() {

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("debug"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_span_events(FmtSpan::ENTER | FmtSpan::EXIT | FmtSpan::CLOSE)
        .init();
}
