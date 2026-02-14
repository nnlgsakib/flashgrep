//! MCP server implementation

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use mcp_server::Server;
use mcp_protocol::types::ToolDefinition;

use crate::mcp::tools::create_tools;

#[derive(Debug, Clone)]
pub struct McpServerConfig {
    pub port: u16,
    pub log_level: String,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            port: 2000,
            log_level: "info".to_string(),
        }
    }
}

pub struct McpServer {
    config: McpServerConfig,
    server: Option<Server>,
}

impl McpServer {
    pub fn new(config: McpServerConfig) -> Self {
        Self {
            config,
            server: None,
        }
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = SocketAddr::from(([127, 0, 0, 1], self.config.port));
        info!("Starting MCP server on {}", addr);

        // Create tools
        let tools = create_tools();
        info!("Registered {} MCP tools", tools.len());

        // Create server
        let server = Server::new(addr, tools).await?;
        self.server = Some(server);

        info!("MCP server started successfully on {}", addr);
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(server) = self.server.take() {
            info!("Shutting down MCP server...");
            server.shutdown().await?;
            info!("MCP server shutdown complete");
        }

        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.server.is_some()
    }
}

pub async fn run_server(config: McpServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    let mut server = McpServer::new(config);
    server.start().await?;

    // Wait for shutdown signal
    let shutdown_signal = tokio::signal::ctrl_c();
    shutdown_signal.await?;

    server.shutdown().await?;
    Ok(())
}