use std::net::SocketAddr;

use anyhow;
use axum::routing::post;
use clap::Parser;
use logag::event_aggregator::EventAggregator;
use logag::global_config::GlobalConfig;
use logag::http_response_handler::HTTPResponseHandler;
use logag::observability::Observability;
use rmcp::transport::StreamableHttpServerConfig;
use rmcp::transport::streamable_http_server::StreamableHttpService;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use tokio;

const MCP_BIND_ADDR: &str = "127.0.0.1:8000";

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
    let args = CmdArgs::parse();

    let config = GlobalConfig::load_config(args.config_path.to_string());

    Observability::init();

    let aggregator = EventAggregator::new(config);
    let mcp_aggregator = aggregator.clone();

    let mcp_service = StreamableHttpService::new(
        move || Ok(mcp_aggregator.clone()),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default(),
    );

    let http_handler = std::sync::Arc::new(HTTPResponseHandler::new(aggregator));

    let router: axum::Router = axum::Router::new()
        .nest_service("/mcp", mcp_service)
        .route("/record", post(HTTPResponseHandler::handle_post_response))
        .with_state(http_handler);
    let tcp_listener = tokio::net::TcpListener::bind(MCP_BIND_ADDR).await?;
    tracing::info!("Started HTTP + MCP endpoints at {}", MCP_BIND_ADDR);
    let _ = axum::serve(tcp_listener, router).await;

    Ok(())
}
