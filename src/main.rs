use anyhow::Context as _;
use tracing_subscriber::{EnvFilter, fmt};
use web_rs::{app, config};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    install_tracing()?;
    let app_config = config::load_embedded().context("failed to load embedded config")?;
    app::run(app_config).await.context("server failed")?;
    Ok(())
}
fn install_tracing() -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt()
        .with_env_filter(filter)
        .try_init()
        .map_err(|error| anyhow::anyhow!("failed to install tracing subscriber: {error}"))?;
    Ok(())
}
