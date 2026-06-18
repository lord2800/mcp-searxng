mod config;
mod http_client;
mod mcp_server;
mod model;
mod query_builder;
mod search_service;

use clap::Parser;
use rmcp::ServiceExt;
use tokio::signal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = config::AppConfig::parse();

    // Set up graceful shutdown signal.
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let shutdown_tx_clone = shutdown_tx.clone();

    tokio::spawn(async move {
        // Wait for Ctrl-C or SIGTERM
        let _ = signal::ctrl_c().await;
        let _ = shutdown_tx_clone.send(true);
    });

    // Create the MCP server.
    let server = mcp_server::McpServer::new(config.clone());

    match config.transport {
        config::Transport::Stdio => run_stdio(server, shutdown_rx).await,
        config::Transport::HTTP => run_http(server, &config, shutdown_rx).await,
    }
}

async fn run_stdio(
    server: mcp_server::McpServer,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
) -> anyhow::Result<()> {
    use rmcp::transport::stdio;
    let service = server.serve(stdio()).await?;

    // Run with graceful shutdown.
    tokio::select! {
        result = service.waiting() => {
            result?;
        }
        _ = shutdown_rx.changed() => {
            eprintln!("Shutting down...");
        }
    }

    Ok(())
}

async fn run_http(
    server: mcp_server::McpServer,
    config: &config::AppConfig,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
) -> anyhow::Result<()> {
    use rmcp::transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    };

    let http_config = StreamableHttpServerConfig::default();

    // Build the HTTP service from our MCP server.
    let http_service: StreamableHttpService<_, LocalSessionManager> = StreamableHttpService::new(
        move || Ok(server.clone()),
        std::sync::Arc::new(LocalSessionManager::default()),
        http_config,
    );

    // Wire up with axum router and serve.
    let router = axum::Router::new().nest_service("/mcp", http_service);

    let addr = format!("{}:{}", config.bind_addr, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    eprintln!("HTTP server listening on {}", addr);

    // Spawn the HTTP server and wait for shutdown signal or completion.
    let mut server_handle = tokio::spawn(async move { axum::serve(listener, router).await });

    tokio::select! {
        Ok(result) = &mut server_handle => {
            // Server finished — propagate any IO error.
            result?;
        }
        _ = shutdown_rx.changed() => {
            eprintln!("Shutting down...");
            server_handle.abort();
        }
    }

    Ok(())
}
