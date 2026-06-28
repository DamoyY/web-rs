use crate::{
    Result,
    config::FindConfig,
    error::AppError,
    models::{FindPage, FindRequest, FindResponse, OpenPage, OpenRequest, OpenResponse},
    page::{PageContent, TokenChunker, find_in_page, open_page_chunk},
};
use fancy_regex::Regex;
use futures::future::try_join_all;
pub(crate) async fn open_pages(
    requests: &[OpenRequest],
    pages: Vec<PageContent>,
    chunker: TokenChunker,
    mut warnings: Vec<String>,
) -> Result<OpenResponse> {
    let tasks = requests
        .iter()
        .zip(pages)
        .enumerate()
        .map(|(request_index, (request, page))| {
            let page_chunker = chunker.clone();
            let chunk_index = request.chunk;
            tokio::task::spawn_blocking(move || {
                let mut page_warnings = Vec::new();
                let opened = open_page_chunk(
                    &page,
                    chunk_index,
                    request_index,
                    &page_chunker,
                    &mut page_warnings,
                )?;
                Ok::<(OpenPage, Vec<String>), AppError>((opened, page_warnings))
            })
        });
    let joined = try_join_all(tasks)
        .await
        .map_err(|error| AppError::internal(format!("page processing task failed: {error}")))?;
    let mut opened = Vec::with_capacity(joined.len());
    for result in joined {
        let (page, page_warnings) = result?;
        opened.push(page);
        warnings.extend(page_warnings);
    }
    Ok(OpenResponse {
        pages: opened,
        warning: (!warnings.is_empty()).then_some(warnings),
    })
}
pub(crate) async fn find_pages(
    requests: &[FindRequest],
    pages: Vec<PageContent>,
    patterns: Vec<Regex>,
    chunker: TokenChunker,
    config: FindConfig,
    chunk_tokens: usize,
    mut warnings: Vec<String>,
) -> Result<FindResponse> {
    let tasks = requests.iter().zip(pages).zip(patterns).enumerate().map(
        |(request_index, ((request, page), pattern))| {
            let page_chunker = chunker.clone();
            let find_config = config.clone();
            let requested_snippet_tokens = request.snippet_tokens;
            tokio::task::spawn_blocking(move || {
                let mut page_warnings = Vec::new();
                let snippet_tokens = snippet_tokens_for_request(
                    requested_snippet_tokens,
                    request_index,
                    &mut page_warnings,
                    chunk_tokens,
                    find_config.default_snippet_tokens,
                );
                let found =
                    find_in_page(&page, &pattern, snippet_tokens, &page_chunker, &find_config)?;
                Ok::<(FindPage, Vec<String>), AppError>((found, page_warnings))
            })
        },
    );
    let joined = try_join_all(tasks)
        .await
        .map_err(|error| AppError::internal(format!("page processing task failed: {error}")))?;
    let mut found = Vec::with_capacity(joined.len());
    for result in joined {
        let (page, page_warnings) = result?;
        found.push(page);
        warnings.extend(page_warnings);
    }
    Ok(FindResponse {
        pages: found,
        warning: (!warnings.is_empty()).then_some(warnings),
    })
}
fn snippet_tokens_for_request(
    requested: Option<usize>,
    request_index: usize,
    warnings: &mut Vec<String>,
    chunk_tokens: usize,
    default_snippet_tokens: usize,
) -> usize {
    let Some(value) = requested else {
        return default_snippet_tokens;
    };
    if value <= chunk_tokens {
        return value;
    }
    warnings . push (format ! ("\"requests[{request_index}].snippet_tokens\" exceeds chunk_tokens ({chunk_tokens}); using {chunk_tokens}")) ;
    chunk_tokens
}
