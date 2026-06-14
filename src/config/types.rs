#![expect(
    clippy::pedantic,
    clippy::restriction,
    reason = "Config structs intentionally mirror embedded YAML keys."
)]
use crate::{Result, error::AppError};
use serde::{Deserialize, Serialize};
use url::Url;
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    pub name: String,
    pub instructions: String,
    pub host: String,
    pub port: u16,
    pub log_level: String,
    pub streamable_http_path: String,
    pub health_path: String,
    pub protocol_version: String,
    pub stateful_http: bool,
    pub json_response: bool,
    pub allowed_hosts: Vec<String>,
    pub allowed_origins: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HeaderConfig {
    pub exa_api_key: String,
    pub jina_api_key: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SearchConfig {
    pub endpoint: String,
    pub num_results: u32,
    #[serde(rename = "type")]
    pub search_type: String,
    pub highlights_max_characters: u32,
    pub max_age_hours: u64,
    pub livecrawl_timeout: u32,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HttpConfig {
    pub timeout_seconds: f64,
    pub direct_fetch_timeout_seconds: f64,
    pub max_redirects: usize,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JinaViewportConfig {
    pub width: u32,
    pub height: u32,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JinaConfig {
    pub endpoint: String,
    pub accept: String,
    pub return_format: String,
    pub engine: String,
    pub locale: String,
    pub no_cache: bool,
    pub respond_with: String,
    pub retain_images: String,
    pub with_shadow_dom: bool,
    pub viewport: JinaViewportConfig,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ChunkingConfig {
    pub tokenizer: String,
    pub chunk_tokens: usize,
    pub overlap_ratio: f64,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FindConfig {
    pub default_snippet_tokens: usize,
    pub max_matches_per_page: usize,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DirectFetchConfig {
    pub max_bytes: usize,
    pub github_hosts: Vec<String>,
    pub huggingface_hosts: Vec<String>,
    pub gitlab_hosts: Vec<String>,
    pub bitbucket_hosts: Vec<String>,
    pub text_file_extensions: Vec<String>,
    pub text_file_names: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SsrfConfig {
    pub block_private_networks: bool,
    pub block_local_hostnames: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub headers: HeaderConfig,
    pub search: SearchConfig,
    pub http: HttpConfig,
    pub jina: JinaConfig,
    pub chunking: ChunkingConfig,
    pub find: FindConfig,
    pub direct_fetch: DirectFetchConfig,
    pub ssrf: SsrfConfig,
}
impl AppConfig {
    pub fn validate(&self) -> Result<()> {
        positive(self.search.num_results, "search.num_results")?;
        positive(
            self.search.highlights_max_characters,
            "search.highlights_max_characters",
        )?;
        positive(self.search.livecrawl_timeout, "search.livecrawl_timeout")?;
        positive(self.chunking.chunk_tokens, "chunking.chunk_tokens")?;
        positive(
            self.find.default_snippet_tokens,
            "find.default_snippet_tokens",
        )?;
        positive(self.find.max_matches_per_page, "find.max_matches_per_page")?;
        positive(self.direct_fetch.max_bytes, "direct_fetch.max_bytes")?;
        positive_float(self.http.timeout_seconds, "http.timeout_seconds")?;
        positive_float(
            self.http.direct_fetch_timeout_seconds,
            "http.direct_fetch_timeout_seconds",
        )?;
        if !(0.0..1.0).contains(&self.chunking.overlap_ratio) {
            return Err(AppError::config(
                "chunking.overlap_ratio must be >= 0 and < 1",
            ));
        }
        parse_endpoint(&self.search.endpoint, "search.endpoint")?;
        parse_endpoint(&self.jina.endpoint, "jina.endpoint")?;
        Ok(())
    }
}
fn positive<T>(value: T, path: &str) -> Result<()>
where
    T: PartialOrd + From<u8>,
{
    if value > T::from(0) {
        return Ok(());
    }
    Err(AppError::config(format!("{path} must be positive")))
}
fn positive_float(value: f64, path: &str) -> Result<()> {
    if value.is_finite() && value > 0.0 {
        return Ok(());
    }
    Err(AppError::config(format!("{path} must be positive")))
}
fn parse_endpoint(value: &str, path: &str) -> Result<()> {
    let url = Url::parse(value).map_err(|error| AppError::config(format!("{path}: {error}")))?;
    if matches!(url.scheme(), "http" | "https") && url.host_str().is_some() {
        return Ok(());
    }
    Err(AppError::config(format!(
        "{path} must be an absolute HTTP or HTTPS URL"
    )))
}
