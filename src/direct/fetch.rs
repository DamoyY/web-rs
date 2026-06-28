use crate::{
    Result,
    config::{DirectFetchConfig, HttpConfig},
    direct::{
        content::extract_content,
        target::{DirectFetchTarget, ResponseFormat},
    },
    error::AppError,
    net::{FetchResponse, SecureHttpClient},
};
use futures::future::{BoxFuture, FutureExt as _, Shared, join};
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue, USER_AGENT};
pub type SharedProbeFetch = Shared<BoxFuture<'static, Result<FetchResponse>>>;
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
    let headers = request_headers(client, target)?;
    let (response, probe_response) =
        fetch_target_responses(client, target, headers, direct_config, http_config).await?;
    extract_direct_content(target, &response, probe_response, direct_config)
}
#[expect(
    clippy::missing_inline_in_public_items,
    reason = "The public direct fetch entrypoint performs HTTP I/O and keeps protocol terminology."
)]
pub async fn fetch_direct_text_with_probe(
    client: &SecureHttpClient,
    target: &DirectFetchTarget,
    direct_config: &DirectFetchConfig,
    http_config: &HttpConfig,
    probe_fetch: SharedProbeFetch,
) -> Result<String> {
    let headers = request_headers(client, target)?;
    let main_fetch = fetch_response(
        client,
        &target.request_url,
        headers,
        direct_config,
        http_config,
    );
    let (response_result, probe_result) = join(main_fetch, probe_fetch).await;
    let response = response_result?;
    extract_direct_content(target, &response, Some(probe_result), direct_config)
}
#[inline]
pub fn shared_probe_fetch(
    client: SecureHttpClient,
    probe_url: String,
    target: &DirectFetchTarget,
    direct_config: &DirectFetchConfig,
    http_config: &HttpConfig,
) -> Result<SharedProbeFetch> {
    let headers = request_headers(&client, target)?;
    let timeout_seconds = http_config.direct_fetch_timeout_seconds;
    let max_bytes = direct_config.max_bytes;
    Ok(async move {
        client
            .get_with_body_limit(&probe_url, headers, timeout_seconds, max_bytes)
            .await
    }
    .boxed()
    .shared())
}
fn extract_direct_content(
    target: &DirectFetchTarget,
    response: &FetchResponse,
    probe_response: Option<Result<FetchResponse>>,
    direct_config: &DirectFetchConfig,
) -> Result<String> {
    let content = extract_content(
        target,
        response.status.as_u16(),
        &response.headers,
        &response.body,
        direct_config,
    )?;
    if response.status.as_u16() == 200
        && let Some(probe_result) = probe_response
    {
        let probe_check_response = probe_result?;
        reject_if_probe_is_similar(target, &probe_check_response, direct_config, &content)?;
    }
    Ok(content)
}
async fn fetch_target_responses(
    client: &SecureHttpClient,
    target: &DirectFetchTarget,
    headers: HeaderMap,
    direct_config: &DirectFetchConfig,
    http_config: &HttpConfig,
) -> Result<(FetchResponse, Option<Result<FetchResponse>>)> {
    let Some(probe_url) = target.similarity_probe_url.as_deref() else {
        return Ok((
            fetch_response(
                client,
                &target.request_url,
                headers,
                direct_config,
                http_config,
            )
            .await?,
            None,
        ));
    };
    let main_fetch = fetch_response(
        client,
        &target.request_url,
        headers.clone(),
        direct_config,
        http_config,
    );
    let probe_fetch = fetch_response(client, probe_url, headers, direct_config, http_config);
    let (main_result, probe_result) = join(main_fetch, probe_fetch).await;
    Ok((main_result?, Some(probe_result)))
}
async fn fetch_response(
    client: &SecureHttpClient,
    url: &str,
    headers: HeaderMap,
    direct_config: &DirectFetchConfig,
    http_config: &HttpConfig,
) -> Result<FetchResponse> {
    client
        .get_with_body_limit(
            url,
            headers,
            http_config.direct_fetch_timeout_seconds,
            direct_config.max_bytes,
        )
        .await
}
fn request_headers(client: &SecureHttpClient, target: &DirectFetchTarget) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert(
        ACCEPT,
        HeaderValue::from_str(&accept_header(target)).map_err(header_error)?,
    );
    headers.insert(USER_AGENT, client.user_agent());
    Ok(headers)
}
fn reject_if_probe_is_similar(
    target: &DirectFetchTarget,
    response: &FetchResponse,
    direct_config: &DirectFetchConfig,
    content: &str,
) -> Result<()> {
    let Some(probe_url) = target.similarity_probe_url.as_deref() else {
        return Ok(());
    };
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
#[cfg(test)]
mod tests;
