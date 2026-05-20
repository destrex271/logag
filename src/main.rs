use anyhow;
use rmcp::transport::StreamableHttpServerConfig;
use rmcp::transport::streamable_http_server::StreamableHttpService;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use tokio;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use logag::event_aggregator::EventAggregator;

const BIND_ADDR: &str = "127.0.0.1:8000";

//
// TODO: Add Auth.
// TODO: Add list tools.
//

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // sets tracing based on the environment.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".to_string().into()),
        )
        .init();

    let service = StreamableHttpService::new(
        || Ok(EventAggregator::new()),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default(),
    );

    let router: axum::Router = axum::Router::new().nest_service("/mcp", service);
    let tcp_listener = tokio::net::TcpListener::bind(BIND_ADDR).await?;
    let _ = axum::serve(tcp_listener, router).await;

    Ok(())
}
