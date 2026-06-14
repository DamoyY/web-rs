use crate::{
    Result,
    config::FindConfig,
    models::{FindMatch, FindPage},
    page::{PageContent, TokenChunker},
};
use regex::Regex;
#[expect(
    clippy::missing_inline_in_public_items,
    reason = "Page search loops over chunks and regex matches, so inlining is not useful."
)]
pub fn find_in_page(
    page: &PageContent,
    regex: &Regex,
    snippet_tokens: usize,
    chunker: &TokenChunker,
    config: &FindConfig,
) -> Result<FindPage> {
    let chunks = chunker.split(&page.markdown)?;
    let mut matches = Vec::new();
    for chunk in &chunks {
        for found in regex.find_iter(&chunk.content) {
            matches.push(FindMatch {
                chunk: chunk.index,
                snippet: chunker.snippet_around_span(
                    &chunk.content,
                    found.start(),
                    found.end(),
                    snippet_tokens,
                )?,
            });
            if matches.len() >= config.max_matches_per_page {
                return Ok(FindPage {
                    total_chunks: chunks.len(),
                    matches,
                });
            }
        }
    }
    Ok(FindPage {
        total_chunks: chunks.len(),
        matches,
    })
}
