#![expect(
    clippy::pedantic,
    clippy::restriction,
    reason = "Token slicing is bounded by tokenizer output and regex byte spans."
)]
use crate::{Result, config::ChunkingConfig, error::AppError};
use tiktoken::CoreBpe;
#[derive(Clone, Debug)]
pub struct TextChunk {
    pub index: usize,
    pub content: String,
}
#[derive(Clone)]
pub struct TokenChunker {
    encoder: &'static CoreBpe,
    chunk_tokens: usize,
    overlap_tokens: usize,
}
impl TokenChunker {
    pub fn new(config: &ChunkingConfig) -> Result<Self> {
        let encoder = tiktoken::get_encoding(&config.tokenizer)
            .ok_or_else(|| AppError::config(format!("unknown tokenizer: {}", config.tokenizer)))?;
        Ok(Self {
            encoder,
            chunk_tokens: config.chunk_tokens,
            overlap_tokens: (config.chunk_tokens as f64 * config.overlap_ratio) as usize,
        })
    }
    #[must_use]
    pub fn count_tokens(&self, text: &str) -> usize {
        self.encoder.encode(text).len()
    }
    pub fn split(&self, text: &str) -> Result<Vec<TextChunk>> {
        let tokens = self.encoder.encode(text);
        if tokens.len() <= self.chunk_tokens {
            return Ok(vec![TextChunk {
                index: 1,
                content: text.to_owned(),
            }]);
        }
        let mut chunks = Vec::new();
        let step = self.chunk_tokens.saturating_sub(self.overlap_tokens).max(1);
        let mut start = 0;
        while start < tokens.len() {
            let end = tokens.len().min(start.saturating_add(self.chunk_tokens));
            chunks.push(TextChunk {
                index: chunks.len() + 1,
                content: self.decode(&tokens[start..end])?,
            });
            if end == tokens.len() {
                break;
            }
            start = start.saturating_add(step);
        }
        Ok(chunks)
    }
    pub fn snippet_around_span(
        &self,
        text: &str,
        start: usize,
        end: usize,
        max_tokens: usize,
    ) -> Result<String> {
        let before_tokens = self.encoder.encode(&text[..start]);
        let match_tokens = self.encoder.encode(&text[start..end]);
        let after_tokens = self.encoder.encode(&text[end..]);
        if match_tokens.len() >= max_tokens {
            return self.decode(&match_tokens[..max_tokens]);
        }
        let remaining = max_tokens.saturating_sub(match_tokens.len());
        let mut left_count = before_tokens.len().min(remaining / 2);
        let mut right_count = after_tokens.len().min(remaining.saturating_sub(left_count));
        let unused = remaining
            .saturating_sub(left_count)
            .saturating_sub(right_count);
        let extra_left = before_tokens.len().saturating_sub(left_count).min(unused);
        left_count = left_count.saturating_add(extra_left);
        right_count = right_count.saturating_add(
            after_tokens
                .len()
                .saturating_sub(right_count)
                .min(unused.saturating_sub(extra_left)),
        );
        let mut selected = tail(&before_tokens, left_count);
        selected.extend_from_slice(&match_tokens);
        selected.extend_from_slice(&after_tokens[..right_count]);
        self.decode(&selected)
    }
    fn decode(&self, tokens: &[u32]) -> Result<String> {
        self.encoder
            .decode_to_string(tokens)
            .map_err(|error| AppError::internal(format!("token decode failed: {error}")))
    }
}
fn tail(items: &[u32], count: usize) -> Vec<u32> {
    if count == 0 {
        return Vec::new();
    }
    items[items.len().saturating_sub(count)..].to_vec()
}
