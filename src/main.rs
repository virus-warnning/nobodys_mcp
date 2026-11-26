mod config;
mod server;
mod tools;

use config::Config;
use server::NobodyMcpServer;
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
};
use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("nobodys_mcp=info".parse()?),
        )
        .init();

    // Load configuration
    let config = Config::load_or_default()?;

    info!(
        "Starting {} v{} on {}:{}{}",
        config.server_name, config.version, config.host, config.port, config.endpoint
    );

    // Build the MCP server
    let endpoint = config.endpoint.clone();
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let server = NobodyMcpServer::new(config);

    // Create Streamable HTTP service
    let http_config = StreamableHttpServerConfig::default().with_allowed_hosts(vec![
        String::from("localhost"),
        String::from("127.0.0.1"),
        String::from("0.0.0.0"),
    ]);

    let service: StreamableHttpService<NobodyMcpServer, LocalSessionManager> =
        StreamableHttpService::new(move || Ok(server.clone()), Default::default(), http_config);

    // Build axum router
    let router = axum::Router::new().nest_service(&endpoint, service);

    // Start server
    info!("Server listening at http://{addr}{endpoint}");

    axum::serve(tokio::net::TcpListener::bind(addr).await?, router).await?;

    Ok(())
}
