use crate::{
    Result,
    config::AppConfig,
    mcp::{handler::health, http_service},
};
use axum::{Router, routing::get};
use core::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;
#[expect(
    clippy::missing_inline_in_public_items,
    reason = "The async server entrypoint performs network setup and is not an inline candidate."
)]
pub async fn run(config: AppConfig) -> anyhow::Result<()> {
    let address = SocketAddr::new(config.server.host.parse()?, config.server.port);
    let router = router(config.clone()).map_err(anyhow::Error::from)?;
    let listener = TcpListener::bind(address).await?;
    info!("web MCP server listening on http://{address}");
    axum::serve(listener, router).await?;
    Ok(())
}
#[inline]
pub fn router(config: AppConfig) -> Result<Router> {
    let service = http_service(&config)?;
    Ok(Router::new()
        .route(&config.server.health_path, get(health))
        .nest_service(&config.server.streamable_http_path, service)
        .with_state(config))
}
