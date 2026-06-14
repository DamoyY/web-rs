use crate::{
    Result,
    config::ChunkingConfig,
    models::OpenPage,
    page::{PageContent, TokenChunker, fetcher::PageSource, open_page_chunk},
};
#[test]
fn chunker_splits_with_overlap_and_limits_snippet() -> Result<()> {
    let chunker = TokenChunker::new(&ChunkingConfig {
        tokenizer: "o200k_base".to_owned(),
        chunk_tokens: 10,
        overlap_ratio: 0.2,
    })?;
    let text = (0..40)
        .map(|index| format!("word{index}"))
        .collect::<Vec<_>>()
        .join(" ");
    let chunks = chunker.split(&text)?;
    assert!(chunks.len() > 1);
    let snippet = chunker.snippet_around_span("alpha beta gamma", 6, 10, 3)?;
    assert!(snippet.contains("beta"));
    assert!(chunker.count_tokens(&snippet) <= 3);
    Ok(())
}
#[test]
fn open_out_of_range_chunk_uses_first_chunk() -> Result<()> {
    let chunker = TokenChunker::new(&ChunkingConfig {
        tokenizer: "o200k_base".to_owned(),
        chunk_tokens: 100,
        overlap_ratio: 0.1,
    })?;
    let page = PageContent {
        url: "https://example.com".to_owned(),
        source: PageSource::Direct,
        markdown: "alpha beta gamma".to_owned(),
    };
    let mut warnings = Vec::new();
    let opened: OpenPage = open_page_chunk(&page, 2, 0, &chunker, &mut warnings)?;
    assert_eq!(opened.chunk, 1);
    assert_eq!(
        warnings,
        ["\"requests[0].chunk\" must be between 1 and 1; using 1"]
    );
    Ok(())
}
