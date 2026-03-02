use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .from_env_lossy()
                .add_directive("harmony=trace".parse().unwrap()),
        )
        .init();
}
