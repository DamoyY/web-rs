#![expect(
    clippy::pedantic,
    clippy::restriction,
    reason = "Axum entrypoint code follows framework naming and ownership conventions."
)]
use crate::{
    Result,
    config::AppConfig,
    mcp::{
        handler::{health, mcp_entrypoint},
        state,
    },
};
use axum::{
    Router,
    routing::{get, post},
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;
pub async fn run(config: AppConfig) -> anyhow::Result<()> {
    let address = SocketAddr::new(config.server.host.parse()?, config.server.port);
    let router = router(config.clone()).map_err(anyhow::Error::from)?;
    let listener = TcpListener::bind(address).await?;
    info!("web MCP server listening on http://{address}");
    axum::serve(listener, router).await?;
    Ok(())
}
pub fn router(config: AppConfig) -> Result<Router> {
    let state = state(config.clone())?;
    Ok(Router::new()
        .route(&config.server.health_path, get(health))
        .route(&config.server.streamable_http_path, post(mcp_entrypoint))
        .with_state(state))
}
