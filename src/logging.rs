use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logging() {
    // Capture logs from crates still using the `log` crate
    LogTracer::init().expect("Failed to set logger");

    // Set up environment filter, defaulting to "info" if not set
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // Decide between pretty (dev) or JSON (prod) based on env or feature
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true);

    // Change this condition as needed for runtime detection
    #[cfg(debug_assertions)]
    let fmt_layer = fmt_layer.pretty();

    #[cfg(not(debug_assertions))]
    let fmt_layer = fmt_layer.json();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}
