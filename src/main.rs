use std::net::SocketAddr;

use anyhow;
use axum::routing::post;
use clap::Parser;
use logag::event_aggregator::EventAggregator;
use logag::global_config::GlobalConfig;
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

    let mcp_service = StreamableHttpService::new(
        move || Ok(EventAggregator::new(config.clone())), // Clone allocated the string again on
        // the heap. Since string points to a
        // value on heap, it cannot be copy, to
        // avoid dangling pointers.
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default(),
    );

    let router: axum::Router = axum::Router::new()
        .nest_service("/mcp", mcp_service)
        .route("/record", post(handle_post_response));
    let tcp_listener = tokio::net::TcpListener::bind(MCP_BIND_ADDR).await?;
    tracing::info!("Started HTTP + MCP endpoints at {}", MCP_BIND_ADDR);
    let _ = axum::serve(tcp_listener, router).await;


    Ok(())
}

async fn handle_post_response(body: String) -> &'static str {
    tracing::info!("Recieved the following data from Agent: {}", body);
    "Ok"
}
