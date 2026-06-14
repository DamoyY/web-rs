use crate::{
    Result,
    config::{AppConfig, JinaViewportConfig},
    error::{AppError, http_service_error},
    net::SecureHttpClient,
};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::Serialize;
use sonic_rs::{JsonContainerTrait as _, JsonValueTrait as _, Value};
#[cfg(test)]
mod tests;
#[derive(Clone)]
#[expect(
    clippy::module_name_repetitions,
    reason = "The client type name mirrors the upstream Jina Reader service."
)]
pub struct JinaReaderClient {
    config: AppConfig,
    http: SecureHttpClient,
}
#[derive(Serialize)]
struct JinaPayload<'config> {
    url: String,
    viewport: &'config JinaViewportConfig,
}
impl JinaReaderClient {
    #[inline]
    #[must_use]
    pub const fn new(config: AppConfig, http: SecureHttpClient) -> Self {
        Self { config, http }
    }
    #[expect(
        clippy::missing_inline_in_public_items,
        reason = "Jina reads perform async HTTP I/O and are not inline candidates."
    )]
    pub async fn read_markdown(&self, url: &str, api_key: &str) -> Result<String> {
        let headers = self.headers(api_key)?;
        let payload = JinaPayload {
            url: rewrite_arxiv_pdf_url(url),
            viewport: &self.config.jina.viewport,
        };
        let body = sonic_rs::to_vec(&payload).map_err(|error| {
            AppError::internal(format!("failed to encode Jina request: {error}"))
        })?;
        let response = self
            .http
            .post(
                &self.config.jina.endpoint,
                headers,
                body,
                self.config.http.timeout_seconds,
            )
            .await?;
        if response.status.as_u16() >= 400 {
            return Err(http_service_error("Jina", response.status.as_u16()));
        }
        extract_content(&response.headers, &response.body)
    }
    fn headers(&self, api_key: &str) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {api_key}")).map_err(header_error)?,
        );
        headers.insert(
            ACCEPT,
            HeaderValue::from_str(&self.config.jina.accept).map_err(header_error)?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        insert_header(&mut headers, "X-Engine", &self.config.jina.engine)?;
        insert_header(&mut headers, "X-Locale", &self.config.jina.locale)?;
        insert_header(
            &mut headers,
            "X-No-Cache",
            header_bool(self.config.jina.no_cache),
        )?;
        insert_header(
            &mut headers,
            "X-Respond-With",
            &self.config.jina.respond_with,
        )?;
        insert_header(
            &mut headers,
            "X-Retain-Images",
            &self.config.jina.retain_images,
        )?;
        insert_header(
            &mut headers,
            "X-Return-Format",
            &self.config.jina.return_format,
        )?;
        insert_header(
            &mut headers,
            "X-With-Shadow-Dom",
            header_bool(self.config.jina.with_shadow_dom),
        )?;
        Ok(headers)
    }
}
fn extract_content(headers: &HeaderMap, body: &[u8]) -> Result<String> {
    let content_type = headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .split(';')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if content_type == "text/event-stream" {
        return Ok(extract_event_stream_content(&String::from_utf8_lossy(body)));
    }
    sonic_rs::from_slice::<Value>(body).map_or_else(
        |_error| Ok(String::from_utf8_lossy(body).into_owned()),
        |payload| {
            extract_payload_content(&payload).ok_or_else(|| {
                AppError::client(
                    "Jina returned an unsupported response. Retry later or try another URL.",
                )
            })
        },
    )
}
fn extract_payload_content(payload: &Value) -> Option<String> {
    if let Some(object) = payload.as_object() {
        if let Some(data) = object.get(&"data")
            && let Some(content) = extract_payload_content(data)
        {
            return Some(content);
        }
        for key in ["content", "markdown", "text"] {
            if let Some(text) = object.get(&key).and_then(|value| value.as_str()) {
                return Some(text.to_owned());
            }
        }
    }
    payload.as_str().map(str::to_owned)
}
fn extract_event_stream_content(text: &str) -> String {
    let mut latest_content = None;
    let mut event_lines = Vec::new();
    for line in text.lines() {
        if line.is_empty() {
            latest_content = event_stream_content(latest_content, &event_lines);
            event_lines.clear();
        } else if let Some(data) = line.strip_prefix("data:") {
            event_lines.push(data.strip_prefix(' ').unwrap_or(data).to_owned());
        }
    }
    event_stream_content(latest_content, &event_lines).unwrap_or_else(|| text.to_owned())
}
fn event_stream_content(latest: Option<String>, event_lines: &[String]) -> Option<String> {
    if event_lines.is_empty() {
        return latest;
    }
    let data = event_lines.join("\n");
    if data == "[DONE]" {
        return latest;
    }
    let Ok(payload) = sonic_rs::from_str::<Value>(&data) else {
        return Some(data);
    };
    extract_payload_content(&payload).or(latest)
}
fn insert_header(headers: &mut HeaderMap, name: &'static str, value: &str) -> Result<()> {
    headers.insert(name, HeaderValue::from_str(value).map_err(header_error)?);
    Ok(())
}
const fn header_bool(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}
fn rewrite_arxiv_pdf_url(url: &str) -> String {
    url.strip_prefix("https://arxiv.org/pdf/").map_or_else(
        || url.to_owned(),
        |suffix| format!("https://arxiv.org/html/{suffix}"),
    )
}
#[expect(
    clippy::needless_pass_by_value,
    reason = "map_err passes InvalidHeaderValue by value and the formatter consumes only its Display output."
)]
fn header_error(error: reqwest::header::InvalidHeaderValue) -> AppError {
    AppError::internal(format!("invalid Jina header value: {error}"))
}
