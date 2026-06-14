pub mod client;
pub mod ssrf;
#[cfg(test)]
mod tests;
pub type FetchResponse = client::FetchResponse;
pub type SecureHttpClient = client::SecureHttpClient;
pub type SsrfGuard = ssrf::SsrfGuard;
use crate::config::{HttpConfig, SsrfConfig};
#[must_use]
#[inline]
pub fn secure_client(http: &HttpConfig, ssrf: &SsrfConfig) -> SecureHttpClient {
    SecureHttpClient::new(http.max_redirects, SsrfGuard::new(ssrf.clone()))
}
#[must_use]
#[inline]
pub(crate) fn guard(ssrf: &SsrfConfig) -> SsrfGuard {
    SsrfGuard::new(ssrf.clone())
}
#[must_use]
#[inline]
pub(crate) fn secure_client_from_config(config: &crate::config::AppConfig) -> SecureHttpClient {
    secure_client(&config.http, &config.ssrf)
}
