pub mod code_hosts;
pub mod fetch;
pub mod mediawiki;
pub mod package;
pub mod target;
#[cfg(test)]
mod tests;
use crate::config::DirectFetchConfig;
#[expect(
    clippy::module_name_repetitions,
    reason = "The direct facade re-exports the protocol target name."
)]
pub type DirectFetchTarget = target::DirectFetchTarget;
pub type ResponseFormat = target::ResponseFormat;
#[inline]
#[must_use]
pub fn resolve_direct_fetch_target(
    url: &str,
    config: &DirectFetchConfig,
) -> Option<DirectFetchTarget> {
    let parsed = url::Url::parse(url).ok()?;
    let host = parsed.host_str()?.to_ascii_lowercase();
    if host == "learn.microsoft.com" {
        return Some(DirectFetchTarget::text(
            url,
            microsoft_learn_markdown_url(&parsed),
        ));
    }
    if let Some(raw_url) = code_hosts::resolve_code_host_raw_url(&parsed, &host, config) {
        return Some(DirectFetchTarget::text(url, raw_url));
    }
    if let Some(registry) = package::resolve_package_registry_target(&parsed) {
        return Some(DirectFetchTarget::package(
            url,
            registry.request_url,
            registry.json_fields_last,
        ));
    }
    mediawiki::resolve_mediawiki_api_url(&parsed)
        .map(|request_url| DirectFetchTarget::mediawiki(url, request_url))
}
#[expect(
    clippy::missing_inline_in_public_items,
    reason = "The async direct fetch facade performs HTTP I/O and is not an inline candidate."
)]
pub async fn fetch_direct_text(
    client: &crate::net::SecureHttpClient,
    target: &DirectFetchTarget,
    direct_config: &crate::config::DirectFetchConfig,
    http_config: &crate::config::HttpConfig,
) -> crate::Result<String> {
    fetch::fetch_direct_text(client, target, direct_config, http_config).await
}
fn microsoft_learn_markdown_url(parsed: &url::Url) -> String {
    let mut pairs: Vec<(String, String)> = parsed
        .query_pairs()
        .filter(|pair| !pair.0.as_ref().eq_ignore_ascii_case("accept"))
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect();
    pairs.push(("accept".to_owned(), "text/markdown".to_owned()));
    let mut next = parsed.clone();
    next.query_pairs_mut().clear().extend_pairs(pairs);
    next.to_string()
}
