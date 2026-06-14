use crate::{
    Result,
    config::AppConfig,
    direct::{DirectFetchTarget, fetch_direct_text, resolve_direct_fetch_target},
    error::AppError,
    net::{SecureHttpClient, SsrfGuard, guard, secure_client_from_config},
    page::jina::JinaReaderClient,
};
use tracing::warn;
use url::Url;
const MISSING_MARKDOWN_SUFFIX: &str = ".af97t23bf86rq";
#[derive(Clone, Debug)]
pub struct PageContent {
    pub url: String,
    pub source: PageSource,
    pub markdown: String,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[expect(
    clippy::exhaustive_enums,
    reason = "Page sources are closed because callers branch on the configured readers."
)]
pub enum PageSource {
    Jina,
    Direct,
}
#[derive(Clone)]
pub struct PageFetcher {
    config: AppConfig,
    http: SecureHttpClient,
    guard: SsrfGuard,
    jina: JinaReaderClient,
}
impl PageFetcher {
    #[inline]
    #[must_use]
    pub fn new(config: AppConfig) -> Self {
        let http = secure_client_from_config(&config);
        let guard = guard(&config.ssrf);
        let jina = JinaReaderClient::new(config.clone(), http.clone());
        Self {
            config,
            http,
            guard,
            jina,
        }
    }
    #[expect(
        clippy::missing_inline_in_public_items,
        reason = "Page fetching performs async network I/O and is not an inline candidate."
    )]
    pub async fn fetch(&self, url: &str, jina_api_key: Option<&str>) -> Result<PageContent> {
        let parsed =
            Url::parse(url).map_err(|error| AppError::client(format!("Invalid URL: {error}")))?;
        self.guard.validate_url(&parsed).await?;
        for target in direct_fetch_targets(url, &self.config.direct_fetch) {
            match fetch_direct_text(
                &self.http,
                &target,
                &self.config.direct_fetch,
                &self.config.http,
            )
            .await
            {
                Ok(markdown) => {
                    return Ok(PageContent {
                        url: url.to_owned(),
                        source: PageSource::Direct,
                        markdown,
                    });
                }
                Err(error) => {
                    warn!(
                        "Direct fetch failed for {}: {}",
                        target.request_url,
                        error.client_message()
                    );
                }
            }
        }
        let Some(key) = jina_api_key else {
            return Err(AppError::client(format!(
                "Missing required header: {}. URLs that cannot be directly fetched require a Jina API key.",
                self.config.headers.jina_api_key
            )));
        };
        let markdown = self.jina.read_markdown(url, key).await?;
        Ok(PageContent {
            url: url.to_owned(),
            source: PageSource::Jina,
            markdown,
        })
    }
}
fn direct_fetch_targets(
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
    target.similarity_probe_url = Some(replace_path_suffix(url, MISSING_MARKDOWN_SUFFIX));
    target
}
fn replace_path_suffix(url: &str, suffix: &str) -> String {
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
