use crate::direct::{DirectFetchTarget, resolve_direct_fetch_target};
use url::Url;
pub(super) fn direct_fetch_targets(
    url: &str,
    config: &crate::config::DirectFetchConfig,
) -> Vec<DirectFetchTarget> {
    let markdown_targets = vec![
        markdown_accept_target(url),
        markdown_direct_fetch_target(url),
    ];
    if let Some(target) = resolve_direct_fetch_target(url, config) {
        return core::iter::once(target).chain(markdown_targets).collect();
    }
    markdown_targets
}
fn markdown_accept_target(url: &str) -> DirectFetchTarget {
    DirectFetchTarget::markdown_accept(url)
}
fn markdown_direct_fetch_target(url: &str) -> DirectFetchTarget {
    let mut target = DirectFetchTarget::text(url, replace_path_suffix(url, ".md"));
    target.required_content_type = Some("text/markdown".to_owned());
    target.similarity_probe_url = Some(replace_path_suffix(url, &random_missing_suffix()));
    target
}
pub(super) fn replace_path_suffix(url: &str, suffix: &str) -> String {
    let Ok(mut parsed) = Url::parse(url) else {
        return url.to_owned();
    };
    let path = parsed.path().trim_end_matches('/');
    let next_path = if path.is_empty() {
        format!("/{}", suffix.trim_start_matches('.'))
    } else {
        format!("{path}{suffix}")
    };
    parsed.set_path(&next_path);
    parsed.to_string()
}
fn random_missing_suffix() -> String {
    let value: u128 = rand::random();
    format!(".{value:032x}")
}
