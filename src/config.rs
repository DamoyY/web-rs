#![expect(
    clippy::pedantic,
    clippy::restriction,
    reason = "Config type names intentionally mirror YAML sections."
)]
mod embedded;
#[cfg(test)]
mod tests;
pub mod types;
use crate::Result;
pub type AppConfig = types::AppConfig;
pub type ChunkingConfig = types::ChunkingConfig;
pub type DirectFetchConfig = types::DirectFetchConfig;
pub type FindConfig = types::FindConfig;
pub type HeaderConfig = types::HeaderConfig;
pub type HttpConfig = types::HttpConfig;
pub type JinaConfig = types::JinaConfig;
pub type JinaViewportConfig = types::JinaViewportConfig;
pub type SearchConfig = types::SearchConfig;
pub type ServerConfig = types::ServerConfig;
pub type SsrfConfig = types::SsrfConfig;
#[must_use]
pub const fn default_yaml() -> &'static str {
    embedded::DEFAULT_YAML
}
pub fn load_embedded() -> Result<AppConfig> {
    let config = embedded::load()?;
    config.validate()?;
    Ok(config)
}
