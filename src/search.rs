pub mod exa;
pub type ExaSearchClient = exa::ExaSearchClient;
use crate::config::AppConfig;
#[must_use]
#[inline]
pub(crate) fn client(config: &AppConfig) -> ExaSearchClient {
    ExaSearchClient::new(config)
}
#[must_use]
#[inline]
pub(crate) const fn provider_name() -> &'static str {
    "Exa"
}
#[must_use]
#[inline]
pub(crate) const fn api_key_header() -> &'static str {
    "x-api-key"
}
