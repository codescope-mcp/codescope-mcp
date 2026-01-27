use anyhow::Result;
use rmcp::{transport::stdio, ServiceExt};
use tracing_subscriber::{self, EnvFilter};

use codescope_mcp::server::handler::CodeScopeServer;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing with stderr output (stdout is used for MCP protocol)
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting CodeScope MCP server");

    // Create and serve the MCP server
    let server = CodeScopeServer::new();
    let service = server.serve(stdio()).await.inspect_err(|e| {
        tracing::error!("Server error: {:?}", e);
    })?;

    // Wait for the service to complete
    service.waiting().await?;

    tracing::info!("CodeScope MCP server stopped");
    Ok(())
}
