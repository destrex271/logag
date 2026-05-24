use anyhow;
use clap::Parser;
use logag::RUNTIME;
use logag::global_config::GlobalConfig;
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

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct CmdArgs {
    #[arg(short, long, required = true)]
    config_path: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    // Capture runtime first, without runtime nothing works.
    // This prevents modules from creating their own runtimes.
    // If this leads to pessimistic locking, we should make a 
    // runtime pool sort of implementation to avoid runtime explosion
    // across the codebase.
    let runtime = tokio::runtime::Runtime::new().unwrap();
    RUNTIME.set(runtime).unwrap();

    let args = CmdArgs::parse();

    let config = GlobalConfig::load_config(args.config_path.to_string());

    // sets tracing based on the environment.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".to_string().into()),
        )
        .init();

    let service = StreamableHttpService::new(
        move || Ok(EventAggregator::new(config.clone())), // Clone allocated the string again on
                                                          // the heap. Since string points to a
                                                          // value on heap, it cannot be copy, to
                                                          // avoid dangling pointers.
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default(),
    );

    let router: axum::Router = axum::Router::new().nest_service("/mcp", service);
    let tcp_listener = tokio::net::TcpListener::bind(BIND_ADDR).await?;
    let _ = axum::serve(tcp_listener, router).await;

    Ok(())
}
