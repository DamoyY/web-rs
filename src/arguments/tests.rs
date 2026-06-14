use crate::{
    Result,
    arguments::{open_arguments, search_arguments},
};
use sonic_rs::json;
#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "The test uses assertions while Result keeps setup failures readable."
)]
fn open_accepts_direct_object_with_aliases() -> Result<()> {
    let result = open_arguments(Some(json ! ({ "URL" : "example.com" , "CHUNKS" : 1_u64 })))?;
    let request = result
        .value
        .requests
        .first()
        .ok_or_else(|| crate::error::AppError::internal("missing normalized request"))?;
    assert_eq!(request.url, "https://example.com");
    assert_eq!(request.chunk, 1);
    let warning = result.warning.unwrap_or_default().join("\n");
    assert!(warning.contains("wrap the request object"));
    assert!(warning.contains("use \"url\" instead of \"URL\""));
    assert!(warning.contains("include a URL scheme"));
    Ok(())
}
#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "The test uses assertions while Result keeps setup failures readable."
)]
fn search_wraps_domains_and_normalizes_category() -> Result<()> {
    let result = search_arguments(Some(
        json ! ({ "Request" : { "Query" : "OpenAI" , "Domain" : "openai.com" , "Category" : "Research Papers" } }),
    ))?;
    let request = result
        .value
        .requests
        .first()
        .ok_or_else(|| crate::error::AppError::internal("missing normalized request"))?;
    assert_eq!(request.q, "OpenAI");
    assert_eq!(
        request.domains.as_deref(),
        Some(&["openai.com".to_owned()][..])
    );
    let warning = result.warning.unwrap_or_default().join("\n");
    assert!(warning.contains("pass \"requests[0].domains\" as an array"));
    assert!(warning.contains("use \"research paper\" instead of \"Research Papers\""));
    Ok(())
}
