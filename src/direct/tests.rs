use crate::{
    Result, config,
    direct::{ResponseFormat, resolve_direct_fetch_target},
};
#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "The test uses assertions while Result keeps setup failures readable."
)]
fn github_blob_resolves_to_raw_text_url() -> Result<()> {
    let config = config::load_embedded()?;
    let target = resolve_direct_fetch_target(
        "https://github.com/modelcontextprotocol/python-sdk/blob/main/README.md",
        &config.direct_fetch,
    )
    .ok_or_else(|| crate::error::AppError::internal("github target was not resolved"))?;
    assert_eq!(
        target.request_url,
        "https://raw.githubusercontent.com/modelcontextprotocol/python-sdk/main/README.md"
    );
    assert_eq!(target.response_format, ResponseFormat::Text);
    Ok(())
}
#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "The test uses assertions while Result keeps setup failures readable."
)]
fn scoped_npm_package_uses_registry_json() -> Result<()> {
    let config = config::load_embedded()?;
    let target = resolve_direct_fetch_target(
        "https://www.npmjs.com/package/@types/node",
        &config.direct_fetch,
    )
    .ok_or_else(|| crate::error::AppError::internal("npm target was not resolved"))?;
    assert_eq!(
        target.request_url,
        "https://registry.npmjs.org/@types%2Fnode"
    );
    assert_eq!(target.json_fields_last, ["versions"]);
    Ok(())
}
