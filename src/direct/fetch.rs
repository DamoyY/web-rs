use crate::{
    Result, VERSION,
    config::{DirectFetchConfig, HttpConfig},
    direct::{
        mediawiki::extract_mediawiki_content,
        package::format_package_registry_json,
        stack_overflow::format_stack_overflow_question_json,
        target::{DirectFetchTarget, ResponseFormat},
    },
    error::{AppError, http_service_error},
    net::SecureHttpClient,
};
use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderMap, HeaderValue, RANGE, USER_AGENT};
use sonic_rs::Value;
const SIMILAR_CONTENT_THRESHOLD: f64 = 0.9;
#[expect(
    clippy::missing_inline_in_public_items,
    reason = "The public direct fetch entrypoint performs HTTP I/O and keeps protocol terminology."
)]
pub async fn fetch_direct_text(
    client: &SecureHttpClient,
    target: &DirectFetchTarget,
    direct_config: &DirectFetchConfig,
    http_config: &HttpConfig,
) -> Result<String> {
    let headers = request_headers(target, direct_config)?;
    let response = client
        .get(
            &target.request_url,
            headers.clone(),
            http_config.direct_fetch_timeout_seconds,
        )
        .await?;
    let content = extract_content(
        target,
        response.status.as_u16(),
        &response.headers,
        &response.body,
        direct_config,
    )?;
    if target.similarity_probe_url.is_some() && response.status.as_u16() == 200 {
        reject_if_probe_is_similar(
            client,
            target,
            headers,
            direct_config,
            http_config,
            &content,
        )
        .await?;
    }
    Ok(content)
}
fn request_headers(target: &DirectFetchTarget, config: &DirectFetchConfig) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert(
        ACCEPT,
        HeaderValue::from_str(&accept_header(target)).map_err(header_error)?,
    );
    headers.insert(
        RANGE,
        HeaderValue::from_str(&format!("bytes=0-{}", config.max_bytes)).map_err(header_error)?,
    );
    headers.insert(
        USER_AGENT,
        HeaderValue::from_str(&format!("web-mcp/{VERSION}")).map_err(header_error)?,
    );
    Ok(headers)
}
fn extract_content(
    target: &DirectFetchTarget,
    status_code: u16,
    headers: &HeaderMap,
    body: &[u8],
    direct_config: &DirectFetchConfig,
) -> Result<String> {
    if status_code >= 400 {
        return Err(http_service_error("direct fetch", status_code));
    }
    if body.len() > direct_config.max_bytes {
        return Err(AppError::client(format!(
            "Direct content is larger than the allowed {} bytes.",
            direct_config.max_bytes
        )));
    }
    ensure_required_content_type(target, headers)?;
    match target.response_format {
        ResponseFormat::Text => Ok(String::from_utf8_lossy(body).into_owned()),
        ResponseFormat::MediaWikiApi => {
            let payload = json_payload(body, target.response_format)?;
            extract_mediawiki_content(&payload)
        }
        ResponseFormat::PackageRegistryJson => {
            let payload = json_payload(body, target.response_format)?;
            format_package_registry_json(&payload, &target.json_fields_last)
        }
        ResponseFormat::StackOverflowQuestionJson => {
            let payload = json_payload(body, target.response_format)?;
            format_stack_overflow_question_json(&payload)
        }
    }
}
fn json_payload(body: &[u8], format: ResponseFormat) -> Result<Value> {
    sonic_rs::from_slice(body).map_err(|_error| {
        AppError::client(format!(
            "{} returned malformed JSON.",
            json_service_name(format)
        ))
    })
}
async fn reject_if_probe_is_similar(
    client: &SecureHttpClient,
    target: &DirectFetchTarget,
    headers: HeaderMap,
    direct_config: &DirectFetchConfig,
    http_config: &HttpConfig,
    content: &str,
) -> Result<()> {
    let Some(probe_url) = target.similarity_probe_url.as_deref() else {
        return Ok(());
    };
    let response = client
        .get(probe_url, headers, http_config.direct_fetch_timeout_seconds)
        .await?;
    if response.status.as_u16() != 200 {
        return Ok(());
    }
    let mut probe_target = target.clone();
    #[expect(
        clippy::assigning_clones,
        reason = "The cloned probe URL replaces a cloned request target for one validation request."
    )]
    {
        probe_target.request_url = probe_url.to_owned();
    }
    probe_target.similarity_probe_url = None;
    let probe_content = extract_content(
        &probe_target,
        response.status.as_u16(),
        &response.headers,
        &response.body,
        direct_config,
    )?;
    let similarity = strsim::normalized_levenshtein(content, &probe_content);
    if similarity >= SIMILAR_CONTENT_THRESHOLD {
        return Err(AppError::client(format!(
            "Direct Markdown content is too similar to a known-missing URL response ({similarity:.3})."
        )));
    }
    Ok(())
}
fn ensure_required_content_type(target: &DirectFetchTarget, headers: &HeaderMap) -> Result<()> {
    let Some(expected) = target.required_content_type.as_deref() else {
        return Ok(());
    };
    let Some(content_type) = headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
    else {
        return Err(AppError::client(format!(
            "Direct fetch returned no Content-Type header; expected {expected}."
        )));
    };
    let actual = content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    if actual == expected.to_ascii_lowercase() {
        return Ok(());
    }
    Err(AppError::client(format!(
        "Direct fetch returned Content-Type {content_type}; expected {expected}."
    )))
}
fn accept_header(target: &DirectFetchTarget) -> String {
    if let Some(value) = target.accept_header.clone() {
        return value;
    }
    if matches!(
        target.response_format,
        ResponseFormat::MediaWikiApi
            | ResponseFormat::PackageRegistryJson
            | ResponseFormat::StackOverflowQuestionJson
    ) {
        return "application/json".to_owned();
    }
    "text/plain,*/*".to_owned()
}
const fn json_service_name(format: ResponseFormat) -> &'static str {
    match format {
        ResponseFormat::MediaWikiApi => "MediaWiki API",
        ResponseFormat::PackageRegistryJson => "Package registry",
        ResponseFormat::StackOverflowQuestionJson => "Stack Exchange API",
        ResponseFormat::Text => "direct fetch",
    }
}
#[expect(
    clippy::needless_pass_by_value,
    reason = "map_err passes InvalidHeaderValue by value and the formatter consumes only its Display output."
)]
fn header_error(error: reqwest::header::InvalidHeaderValue) -> AppError {
    AppError::internal(format!("invalid configured HTTP header: {error}"))
}
