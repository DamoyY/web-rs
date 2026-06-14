use crate::{
    Result,
    arguments::{find_arguments, open_arguments, search_arguments},
    config::AppConfig,
    error::AppError,
    models::{FindResponse, OpenResponse, SearchQueryResponse},
    page::{PageFetcher, TokenChunker, find_in_page, open_page_chunk},
    search::ExaSearchClient,
};
use axum::http::HeaderMap;
use futures::future::try_join_all;
use regex::Regex;
use sonic_rs::Value;
#[derive(Clone)]
pub struct ToolService {
    config: AppConfig,
    chunker: TokenChunker,
    page_fetcher: PageFetcher,
    search: ExaSearchClient,
}
impl ToolService {
    #[inline]
    pub fn new(config: AppConfig) -> Result<Self> {
        Ok(Self {
            chunker: TokenChunker::new(&config.chunking)?,
            page_fetcher: PageFetcher::new(config.clone()),
            search: crate::search::client(&config),
            config,
        })
    }
    #[must_use]
    pub(crate) const fn config(&self) -> &AppConfig {
        &self.config
    }
    #[expect(
        clippy::missing_inline_in_public_items,
        reason = "Tool dispatch performs async service work and is not an inline candidate."
    )]
    pub async fn call(
        &self,
        name: &str,
        arguments: Option<Value>,
        headers: &HeaderMap,
    ) -> Result<Value> {
        match name {
            "search_query" => self.search_query(arguments, headers).await,
            "open" => self.open(arguments, headers).await,
            "find" => self.find(arguments, headers).await,
            other => Err(AppError::client(format!("Unknown tool: {other}"))),
        }
    }
    async fn search_query(&self, arguments: Option<Value>, headers: &HeaderMap) -> Result<Value> {
        let normalized = search_arguments(arguments)?;
        let key = required_header(headers, &self.config.headers.exa_api_key)?;
        let results = self
            .search
            .search_many(&normalized.value.requests, &key)
            .await?;
        to_value(&SearchQueryResponse {
            results,
            warning: normalized.warning,
        })
    }
    async fn open(&self, arguments: Option<Value>, headers: &HeaderMap) -> Result<Value> {
        let normalized = open_arguments(arguments)?;
        let mut warnings = normalized.warning.unwrap_or_default();
        let jina_key = optional_header(headers, &self.config.headers.jina_api_key);
        let fetches = normalized
            .value
            .requests
            .iter()
            .map(|request| self.page_fetcher.fetch(&request.url, jina_key.as_deref()));
        let pages = try_join_all(fetches).await?;
        let mut opened = Vec::with_capacity(pages.len());
        for (index, (request, page)) in normalized
            .value
            .requests
            .iter()
            .zip(pages.iter())
            .enumerate()
        {
            opened.push(open_page_chunk(
                page,
                request.chunk,
                index,
                &self.chunker,
                &mut warnings,
            )?);
        }
        to_value(&OpenResponse {
            pages: opened,
            warning: (!warnings.is_empty()).then_some(warnings),
        })
    }
    async fn find(&self, arguments: Option<Value>, headers: &HeaderMap) -> Result<Value> {
        let normalized = find_arguments(arguments)?;
        let jina_key = optional_header(headers, &self.config.headers.jina_api_key);
        let patterns = compile_patterns(&normalized.value.requests)?;
        let fetches = normalized
            .value
            .requests
            .iter()
            .map(|request| self.page_fetcher.fetch(&request.url, jina_key.as_deref()));
        let pages = try_join_all(fetches).await?;
        let mut warnings = normalized.warning.unwrap_or_default();
        let mut found = Vec::with_capacity(pages.len());
        for (index, ((request, page), pattern)) in normalized
            .value
            .requests
            .iter()
            .zip(pages.iter())
            .zip(patterns.iter())
            .enumerate()
        {
            let snippet_tokens = snippet_tokens_for_request(
                request.snippet_tokens,
                index,
                &mut warnings,
                &self.config,
            );
            found.push(find_in_page(
                page,
                pattern,
                snippet_tokens,
                &self.chunker,
                &self.config.find,
            )?);
        }
        to_value(&FindResponse {
            pages: found,
            warning: (!warnings.is_empty()).then_some(warnings),
        })
    }
}
fn compile_patterns(requests: &[crate::models::FindRequest]) -> Result<Vec<Regex>> {
    requests
        .iter()
        .map(|request| {
            Regex::new(&format!("(?m){}", request.pattern)).map_err(|error| {
                AppError::client(format!(
                    "pattern is not a valid regular expression: {error}"
                ))
            })
        })
        .collect()
}
fn snippet_tokens_for_request(
    requested: Option<usize>,
    request_index: usize,
    warnings: &mut Vec<String>,
    config: &AppConfig,
) -> usize {
    let Some(value) = requested else {
        return config.find.default_snippet_tokens;
    };
    if value <= config.chunking.chunk_tokens {
        return value;
    }
    warnings.push(format!(
        "\"requests[{request_index}].snippet_tokens\" exceeds chunk_tokens ({}); using {}",
        config.chunking.chunk_tokens, config.chunking.chunk_tokens
    ));
    config.chunking.chunk_tokens
}
fn required_header(headers: &HeaderMap, name: &str) -> Result<String> {
    optional_header(headers, name)
        .ok_or_else(|| AppError::client(format!("Missing required header: {name}.")))
}
fn optional_header(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}
fn to_value<T>(value: &T) -> Result<Value>
where
    T: serde::Serialize,
{
    sonic_rs::to_value(value)
        .map_err(|error| AppError::internal(format!("failed to encode tool response: {error}")))
}
