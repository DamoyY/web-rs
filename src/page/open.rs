use crate::{
    Result,
    error::AppError,
    models::OpenPage,
    page::{PageContent, TokenChunker},
};
#[expect(
    clippy::missing_inline_in_public_items,
    clippy::module_name_repetitions,
    reason = "Open-page chunk selection allocates chunks and keeps the response model terminology."
)]
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
        chunks
            .first()
            .ok_or_else(|| AppError::internal("page split produced no chunks"))?
    } else {
        chunks
            .get(chunk_index - 1)
            .ok_or_else(|| AppError::internal("validated page chunk was missing"))?
    };
    Ok(OpenPage {
        chunk: selected.index,
        total_chunks: chunks.len(),
        content: selected.content.clone(),
    })
}
