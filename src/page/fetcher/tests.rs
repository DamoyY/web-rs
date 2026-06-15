use super::{direct_fetch_targets, replace_path_suffix};
use crate::{Result, config, error::AppError};
#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "The test uses assertions while Result keeps setup failures readable."
)]
fn markdown_fallback_targets_require_markdown_content_type() -> Result<()> {
    let config = config::load_embedded()?;
    let targets = direct_fetch_targets(
        "https://support.apple.com/en-us/121115",
        &config.direct_fetch,
    );
    assert_eq!(targets.len(), 2);
    let markdown_target = targets
        .get(1)
        .ok_or_else(|| AppError::internal("markdown target missing"))?;
    assert!(
        targets
            .iter()
            .all(|target| { target.required_content_type.as_deref() == Some("text/markdown") })
    );
    assert_eq!(
        markdown_target.request_url,
        "https://support.apple.com/en-us/121115.md"
    );
    Ok(())
}
#[test]
fn markdown_suffix_preserves_query() {
    assert_eq!(
        replace_path_suffix("https://example.com/path?x=1", ".md"),
        "https://example.com/path.md?x=1"
    );
}
