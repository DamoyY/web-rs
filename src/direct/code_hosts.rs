#![expect(
    clippy::indexing_slicing,
    clippy::missing_inline_in_public_items,
    clippy::pattern_type_mismatch,
    reason = "Code-host URL rewriting checks segment counts before indexing."
)]
use crate::config::DirectFetchConfig;
use url::Url;
#[must_use]
pub fn resolve_code_host_raw_url(
    parsed: &Url,
    host: &str,
    config: &DirectFetchConfig,
) -> Option<String> {
    if contains_host(&config.github_hosts, host) {
        return github_raw_url(parsed, host, config);
    }
    if contains_host(&config.huggingface_hosts, host) {
        return huggingface_raw_url(parsed, host, config);
    }
    if contains_host(&config.gitlab_hosts, host) {
        return gitlab_raw_url(parsed, host, config);
    }
    if contains_host(&config.bitbucket_hosts, host) {
        return bitbucket_raw_url(parsed, host, config);
    }
    None
}
fn github_raw_url(parsed: &Url, host: &str, config: &DirectFetchConfig) -> Option<String> {
    if matches!(
        host,
        "raw.githubusercontent.com" | "gist.githubusercontent.com"
    ) {
        return is_text_path(parsed.path(), config).then(|| parsed.to_string());
    }
    let parts = path_parts(parsed.path());
    if parts.len() < 5 || !matches!(parts[2].as_str(), "blob" | "raw") {
        return None;
    }
    let file_path = parts[4..].join("/");
    is_text_path(&file_path, config).then(|| {
        format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}",
            parts[0], parts[1], parts[3], file_path
        )
    })
}
fn huggingface_raw_url(parsed: &Url, host: &str, config: &DirectFetchConfig) -> Option<String> {
    let parts = path_parts(parsed.path());
    let marker = huggingface_marker_index(&parts)?;
    if parts.len() <= marker + 2 {
        return None;
    }
    let file_path = parts[(marker + 2)..].join("/");
    if !is_text_path(&file_path, config) {
        return None;
    }
    let repo = parts[..marker].join("/");
    Some(format!(
        "https://{host}/{repo}/resolve/{}/{file_path}",
        parts[marker + 1]
    ))
}
fn gitlab_raw_url(parsed: &Url, host: &str, config: &DirectFetchConfig) -> Option<String> {
    let parts = path_parts(parsed.path());
    let dash = parts.iter().position(|part| part == "-")?;
    if parts.len() <= dash + 3 || !matches!(parts[dash + 1].as_str(), "blob" | "raw") {
        return None;
    }
    let file_path = parts[(dash + 3)..].join("/");
    if !is_text_path(&file_path, config) {
        return None;
    }
    Some(format!(
        "https://{host}/{}/-/raw/{}/{file_path}",
        parts[..dash].join("/"),
        parts[dash + 2]
    ))
}
fn bitbucket_raw_url(parsed: &Url, host: &str, config: &DirectFetchConfig) -> Option<String> {
    let parts = path_parts(parsed.path());
    if parts.len() < 5 || !matches!(parts[2].as_str(), "src" | "raw") {
        return None;
    }
    let file_path = parts[4..].join("/");
    is_text_path(&file_path, config).then(|| {
        format!(
            "https://{host}/{}/{}/raw/{}/{}",
            parts[0], parts[1], parts[3], file_path
        )
    })
}
fn huggingface_marker_index(parts: &[String]) -> Option<usize> {
    let minimum = if matches!(
        parts.first().map(String::as_str),
        Some("datasets" | "spaces")
    ) {
        2
    } else {
        1
    };
    parts
        .iter()
        .enumerate()
        .find(|(index, part)| {
            *index >= minimum && matches!(part.as_str(), "blob" | "raw" | "resolve")
        })
        .map(|(index, _)| index)
}
fn is_text_path(path: &str, config: &DirectFetchConfig) -> bool {
    let lower_path = path.to_ascii_lowercase();
    if config
        .text_file_extensions
        .iter()
        .any(|extension| lower_path.ends_with(&extension.to_ascii_lowercase()))
    {
        return true;
    }
    let name = path
        .rsplit('/')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    config
        .text_file_names
        .iter()
        .any(|configured| configured.to_ascii_lowercase() == name)
}
fn path_parts(path: &str) -> Vec<String> {
    path.split('/')
        .filter(|part| !part.is_empty())
        .map(str::to_owned)
        .collect()
}
fn contains_host(hosts: &[String], host: &str) -> bool {
    hosts
        .iter()
        .any(|configured| configured.eq_ignore_ascii_case(host))
}
