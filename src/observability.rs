use std::sync::OnceLock;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub struct Observability;

impl Observability {
    pub fn init() -> &'static Self {
        static INIT: OnceLock<Observability> = OnceLock::new();
        INIT.get_or_init(|| {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer())
                .with(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| "debug".to_string().into()),
                )
                .init();
            Observability
        })
    }
}
