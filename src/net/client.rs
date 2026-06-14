#![expect(
    clippy::pedantic,
    clippy::restriction,
    reason = "HTTP client code keeps redirect validation explicit."
)]
use crate::{Result, error::AppError, net::SsrfGuard};
use reqwest::{
    Method, StatusCode, Url,
    header::{HeaderMap, HeaderValue, LOCATION},
    redirect::Policy,
};
use std::time::Duration;
#[derive(Clone)]
pub struct SecureHttpClient {
    client: reqwest::Client,
    guard: SsrfGuard,
    max_redirects: usize,
}
#[derive(Debug)]
pub struct FetchResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}
impl SecureHttpClient {
    #[must_use]
    pub fn new(max_redirects: usize, guard: SsrfGuard) -> Self {
        let client = reqwest::Client::builder()
            .redirect(Policy::none())
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            client,
            guard,
            max_redirects,
        }
    }
    pub async fn get(
        &self,
        url: &str,
        headers: HeaderMap,
        timeout_seconds: f64,
    ) -> Result<FetchResponse> {
        let parsed =
            Url::parse(url).map_err(|error| AppError::client(format!("Invalid URL: {error}")))?;
        self.request_with_redirects(Method::GET, parsed, headers, None, timeout_seconds)
            .await
    }
    pub async fn post(
        &self,
        url: &str,
        headers: HeaderMap,
        body: Vec<u8>,
        timeout_seconds: f64,
    ) -> Result<FetchResponse> {
        let parsed =
            Url::parse(url).map_err(|error| AppError::client(format!("Invalid URL: {error}")))?;
        self.guard.validate_url(&parsed).await?;
        let response = self
            .client
            .post(parsed)
            .headers(headers)
            .body(body)
            .timeout(duration(timeout_seconds)?)
            .send()
            .await?;
        collect_response(response).await
    }
    async fn request_with_redirects(
        &self,
        method: Method,
        mut url: Url,
        headers: HeaderMap,
        body: Option<Vec<u8>>,
        timeout_seconds: f64,
    ) -> Result<FetchResponse> {
        for redirect_index in 0..=self.max_redirects {
            self.guard.validate_url(&url).await?;
            let response = self
                .client
                .request(method.clone(), url.clone())
                .headers(headers.clone())
                .body(body.clone().unwrap_or_default())
                .timeout(duration(timeout_seconds)?)
                .send()
                .await?;
            if !response.status().is_redirection() {
                return collect_response(response).await;
            }
            let Some(next) = redirect_target(response.headers().get(LOCATION), &url)? else {
                return collect_response(response).await;
            };
            if redirect_index == self.max_redirects {
                return Err(AppError::client("Too many redirects while fetching URL."));
            }
            url = next;
        }
        Err(AppError::internal("redirect loop exited unexpectedly"))
    }
}
fn redirect_target(location: Option<&HeaderValue>, base: &Url) -> Result<Option<Url>> {
    let Some(raw) = location else {
        return Ok(None);
    };
    let value = raw
        .to_str()
        .map_err(|_| AppError::client("Redirect location is not valid UTF-8."))?;
    base.join(value)
        .map(Some)
        .map_err(|error| AppError::client(format!("Redirect location is invalid: {error}")))
}
async fn collect_response(response: reqwest::Response) -> Result<FetchResponse> {
    let status = response.status();
    let headers = response.headers().clone();
    let body = response.bytes().await?.to_vec();
    Ok(FetchResponse {
        status,
        headers,
        body,
    })
}
fn duration(seconds: f64) -> Result<Duration> {
    if seconds.is_finite() && seconds > 0.0 {
        return Ok(Duration::from_secs_f64(seconds));
    }
    Err(AppError::config("HTTP timeout must be positive"))
}
