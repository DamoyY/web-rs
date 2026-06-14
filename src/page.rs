pub mod chunking;
pub mod fetcher;
pub mod finder;
pub mod jina;
pub mod open;
#[cfg(test)]
mod tests;
pub type PageContent = fetcher::PageContent;
pub type PageFetcher = fetcher::PageFetcher;
pub type TextChunk = chunking::TextChunk;
pub type TokenChunker = chunking::TokenChunker;
#[inline]
pub fn find_in_page(
    page: &PageContent,
    regex: &fancy_regex::Regex,
    snippet_tokens: usize,
    chunker: &TokenChunker,
    config: &crate::config::FindConfig,
) -> crate::Result<crate::models::FindPage> {
    finder::find_in_page(page, regex, snippet_tokens, chunker, config)
}
#[inline]
pub fn open_page_chunk(
    page: &PageContent,
    chunk_index: usize,
    request_index: usize,
    chunker: &TokenChunker,
    warnings: &mut Vec<String>,
) -> crate::Result<crate::models::OpenPage> {
    open::open_page_chunk(page, chunk_index, request_index, chunker, warnings)
}
