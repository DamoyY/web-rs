use crate::{
    Result,
    config::{DirectFetchConfig, HttpConfig},
    direct::{
        content::extract_content,
        target::{DirectFetchTarget, ResponseFormat},
    },
    error::AppError,
    net::SecureHttpClient,
};
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue, RANGE, USER_AGENT};
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
    let headers = request_headers(client, target, direct_config)?;
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
fn request_headers(
    client: &SecureHttpClient,
    target: &DirectFetchTarget,
    config: &DirectFetchConfig,
) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert(
        ACCEPT,
        HeaderValue::from_str(&accept_header(target)).map_err(header_error)?,
    );
    headers.insert(
        RANGE,
        HeaderValue::from_str(&format!("bytes=0-{}", config.max_bytes)).map_err(header_error)?,
    );
    headers.insert(USER_AGENT, client.user_agent());
    Ok(headers)
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
    if similarity >= direct_config.similarity_threshold {
        return Err(AppError::client(format!(
            "Direct Markdown content is too similar to a known-missing URL response ({similarity:.3})."
        )));
    }
    Ok(())
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
#[expect(
    clippy::needless_pass_by_value,
    reason = "map_err passes InvalidHeaderValue by value and the formatter consumes only its Display output."
)]
fn header_error(error: reqwest::header::InvalidHeaderValue) -> AppError {
    AppError::internal(format!("invalid configured HTTP header: {error}"))
}
