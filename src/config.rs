mod embedded;
#[cfg(test)]
mod tests;
pub mod types;
use crate::Result;
#[expect(
    clippy::module_name_repetitions,
    reason = "The public config facade re-exports YAML section type names."
)]
pub type AppConfig = types::AppConfig;
#[expect(
    clippy::module_name_repetitions,
    reason = "The public config facade re-exports YAML section type names."
)]
pub type ChunkingConfig = types::ChunkingConfig;
#[expect(
    clippy::module_name_repetitions,
    reason = "The public config facade re-exports YAML section type names."
)]
pub type DirectFetchConfig = types::DirectFetchConfig;
#[expect(
    clippy::module_name_repetitions,
    reason = "The public config facade re-exports YAML section type names."
)]
pub type FindConfig = types::FindConfig;
#[expect(
    clippy::module_name_repetitions,
    reason = "The public config facade re-exports YAML section type names."
)]
pub type HeaderConfig = types::HeaderConfig;
#[expect(
    clippy::module_name_repetitions,
    reason = "The public config facade re-exports YAML section type names."
)]
pub type HttpConfig = types::HttpConfig;
#[expect(
    clippy::module_name_repetitions,
    reason = "The public config facade re-exports YAML section type names."
)]
pub type JinaConfig = types::JinaConfig;
#[expect(
    clippy::module_name_repetitions,
    reason = "The public config facade re-exports YAML section type names."
)]
pub type JinaViewportConfig = types::JinaViewportConfig;
#[expect(
    clippy::module_name_repetitions,
    reason = "The public config facade re-exports YAML section type names."
)]
pub type SearchConfig = types::SearchConfig;
#[expect(
    clippy::module_name_repetitions,
    reason = "The public config facade re-exports YAML section type names."
)]
pub type ServerConfig = types::ServerConfig;
#[expect(
    clippy::module_name_repetitions,
    reason = "The public config facade re-exports YAML section type names."
)]
pub type SsrfConfig = types::SsrfConfig;
#[inline]
#[must_use]
pub const fn default_yaml() -> &'static str {
    embedded::DEFAULT_YAML
}
#[inline]
pub fn load_embedded() -> Result<AppConfig> {
    let config = embedded::load()?;
    config.validate()?;
    Ok(config)
}
