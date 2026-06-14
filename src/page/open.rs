#![expect(
    clippy::pedantic,
    clippy::restriction,
    reason = "Open-page chunk selection uses validated one-based indices."
)]
use crate::{
    Result,
    models::OpenPage,
    page::{PageContent, TokenChunker},
};
pub fn open_page_chunk(
    page: &PageContent,
    chunk_index: usize,
    request_index: usize,
    chunker: &TokenChunker,
    warnings: &mut Vec<String>,
) -> Result<OpenPage> {
    let chunks = chunker.split(&page.markdown)?;
    let selected = if chunk_index == 0 || chunk_index > chunks.len() {
        warnings.push(format!(
            "\"requests[{request_index}].chunk\" must be between 1 and {}; using 1",
            chunks.len()
        ));
        &chunks[0]
    } else {
        &chunks[chunk_index - 1]
    };
    Ok(OpenPage {
        chunk: selected.index,
        total_chunks: chunks.len(),
        content: selected.content.clone(),
    })
}
