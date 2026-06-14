#![expect(
    clippy::indexing_slicing,
    clippy::manual_string_new,
    clippy::missing_inline_in_public_items,
    clippy::needless_pass_by_value,
    clippy::return_and_then,
    clippy::unused_trait_names,
    reason = "MediaWiki URL and JSON traversal validates structure before use."
)]
use crate::{Result, error::AppError};
use sonic_rs::{JsonContainerTrait, JsonValueTrait, Value};
use url::Url;
const WIKIMEDIA_DOMAINS: &[&str] = &[
    "mediawiki.org",
    "wikibooks.org",
    "wikidata.org",
    "wikifunctions.org",
    "wikimedia.org",
    "wikinews.org",
    "wikipedia.org",
    "wikiquote.org",
    "wikisource.org",
    "wikispecies.org",
    "wikiversity.org",
    "wikivoyage.org",
    "wiktionary.org",
];
#[must_use]
pub fn resolve_mediawiki_api_url(parsed: &Url) -> Option<String> {
    let host = parsed.host_str()?.to_ascii_lowercase();
    let (selector, api_path) = if is_wikimedia_host(&host) {
        (wikimedia_selector(parsed)?, "/w/api.php".to_owned())
    } else if is_fandom_host(&host) {
        fandom_selector_and_api_path(parsed)?
    } else {
        return None;
    };
    let mut api = Url::parse(&format!("https://{host}{api_path}")).ok()?;
    append_api_parameters(&mut api, selector);
    Some(api.to_string())
}
pub fn extract_mediawiki_content(payload: &Value) -> Result<String> {
    let object = payload
        .as_object()
        .ok_or_else(|| AppError::client("MediaWiki API returned an invalid response object."))?;
    if let Some(error) = object.get(&"error") {
        return Err(AppError::client(mediawiki_api_error_message(error)));
    }
    let query = object
        .get(&"query")
        .and_then(Value::as_object)
        .ok_or_else(|| AppError::client("MediaWiki API response is missing query results."))?;
    if query.get(&"badrevids").is_some() {
        return Err(AppError::client("MediaWiki revision was not found."));
    }
    let pages = query
        .get(&"pages")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::client("MediaWiki API did not return exactly one page."))?;
    if pages.len() != 1 {
        return Err(AppError::client(
            "MediaWiki API did not return exactly one page.",
        ));
    }
    extract_page_content(&pages[0])
}
fn extract_page_content(page: &Value) -> Result<String> {
    let page_object = page
        .as_object()
        .ok_or_else(|| AppError::client("MediaWiki API returned an invalid page object."))?;
    if page_object.get(&"missing").is_some() {
        return Err(AppError::client("MediaWiki page was not found."));
    }
    let revisions = page_object
        .get(&"revisions")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::client("MediaWiki API response is missing page revisions."))?;
    if revisions.len() != 1 {
        return Err(AppError::client(
            "MediaWiki API response is missing page revisions.",
        ));
    }
    revisions[0]
        .get("slots")
        .and_then(|slots| slots.get("main"))
        .and_then(|main| main.get("content"))
        .and_then(|content| content.as_str())
        .map(str::to_owned)
        .ok_or_else(|| AppError::client("MediaWiki API response is missing page content."))
}
fn wikimedia_selector(parsed: &Url) -> Option<(String, String)> {
    if let Some(title) = parsed.path().strip_prefix("/wiki/") {
        return page_selector(parsed, Some(percent_decode(title)));
    }
    (parsed.path() == "/w/index.php")
        .then(|| first_query_value(parsed, "title"))
        .flatten()
        .and_then(|title| page_selector(parsed, Some(title)))
}
fn fandom_selector_and_api_path(parsed: &Url) -> Option<((String, String), String)> {
    let (prefix, title) = if let Some(article) = fandom_article_path(parsed.path()) {
        article
    } else {
        (
            fandom_index_prefix(parsed.path())?,
            first_query_value(parsed, "title")?,
        )
    };
    Some((
        page_selector(parsed, Some(title))?,
        format!("{prefix}/api.php"),
    ))
}
fn page_selector(parsed: &Url, title: Option<String>) -> Option<(String, String)> {
    if let Some(oldid) = first_query_value(parsed, "oldid") {
        return Some(("revids".to_owned(), oldid));
    }
    if let Some(curid) = first_query_value(parsed, "curid") {
        return Some(("pageids".to_owned(), curid));
    }
    title.map(|value| ("titles".to_owned(), value))
}
fn append_api_parameters(api: &mut Url, selector: (String, String)) {
    let mut pairs = api.query_pairs_mut();
    pairs
        .append_pair("action", "query")
        .append_pair("prop", "revisions")
        .append_pair("rvprop", "content")
        .append_pair("rvslots", "main")
        .append_pair(&selector.0, &selector.1);
    if selector.0 == "titles" {
        pairs.append_pair("redirects", "1");
    }
    pairs
        .append_pair("format", "json")
        .append_pair("formatversion", "2");
}
fn fandom_article_path(path: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 3 && parts[1] == "wiki" {
        return Some(("".to_owned(), percent_decode(&parts[2..].join("/"))));
    }
    if parts.len() >= 4 && parts[2] == "wiki" {
        return Some((
            format!("/{}", parts[1]),
            percent_decode(&parts[3..].join("/")),
        ));
    }
    None
}
fn fandom_index_prefix(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').collect();
    if matches!(parts.as_slice(), ["", "index.php"] | ["", "w", "index.php"]) {
        return Some(String::new());
    }
    (parts.len() == 3 && parts[2] == "index.php").then(|| format!("/{}", parts[1]))
}
fn first_query_value(parsed: &Url, name: &str) -> Option<String> {
    parsed
        .query_pairs()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.into_owned())
}
fn is_wikimedia_host(host: &str) -> bool {
    WIKIMEDIA_DOMAINS
        .iter()
        .any(|domain| host == *domain || host.ends_with(&format!(".{domain}")))
}
fn is_fandom_host(host: &str) -> bool {
    host != "www.fandom.com" && host.ends_with(".fandom.com")
}
fn percent_decode(value: &str) -> String {
    percent_encoding::percent_decode_str(value)
        .decode_utf8_lossy()
        .into_owned()
}
fn mediawiki_api_error_message(error: &Value) -> String {
    if let Some(code) = error.get("code").and_then(|value| value.as_str()) {
        return format!("MediaWiki API rejected the page request ({code}).");
    }
    "MediaWiki API rejected the page request.".to_owned()
}
