use crate::{
    Result,
    arguments::{open_arguments, search_arguments},
};
use sonic_rs::{Value, json};
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
#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "The test uses assertions while Result keeps setup failures readable."
)]
fn search_accepts_single_string_under_unknown_nesting() -> Result<()> {
    let result = search_arguments(Some(
        json ! ({ "outer" : [{ "inner" : { "term" : "rust async traits" } }] }),
    ))?;
    let request = result
        .value
        .requests
        .first()
        .ok_or_else(|| crate::error::AppError::internal("missing normalized request"))?;
    assert_eq!(request.q, "rust async traits");
    assert_eq!(request.domains, None);
    Ok(())
}
#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "The test uses assertions while Result keeps setup failures readable."
)]
fn search_accepts_search_terms_alias_and_null_optionals() -> Result<()> {
    let result = search_arguments(Some(
        json ! ({ "requests" : [{ "search_terms" : "mcp rust" , "recency" : null , "domains" : null , "category" : null }] }),
    ))?;
    let request = result
        .value
        .requests
        .first()
        .ok_or_else(|| crate::error::AppError::internal("missing normalized request"))?;
    assert_eq!(request.q, "mcp rust");
    assert_eq!(request.recency, None);
    assert_eq!(request.domains, None);
    assert!(request.category.is_none());
    Ok(())
}
#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "The test uses assertions while Result keeps setup failures readable."
)]
fn open_accepts_single_string_under_unknown_nesting() -> Result<()> {
    let result = open_arguments(Some(
        json ! ({ "request" : { "nested" : ["example.com"] } }),
    ))?;
    let request = result
        .value
        .requests
        .first()
        .ok_or_else(|| crate::error::AppError::internal("missing normalized request"))?;
    assert_eq!(request.url, "https://example.com");
    assert_eq!(request.chunk, 0);
    Ok(())
}
#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "The test uses assertions while Result keeps setup failures readable."
)]
fn search_accepts_plain_string_argument() -> Result<()> {
    let result = search_arguments(Some(Value::from("single search")))?;
    let request = result
        .value
        .requests
        .first()
        .ok_or_else(|| crate::error::AppError::internal("missing normalized request"))?;
    assert_eq!(request.q, "single search");
    Ok(())
}
#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "The test uses assertions while Result keeps setup failures readable."
)]
fn search_extracts_nested_primary_field_value() -> Result<()> {
    let result = search_arguments(Some(
        json ! ({ "requests" : [{ "q" : { "payload" : [{ "text" : "nested query" }] } }] }),
    ))?;
    let request = result
        .value
        .requests
        .first()
        .ok_or_else(|| crate::error::AppError::internal("missing normalized request"))?;
    assert_eq!(request.q, "nested query");
    Ok(())
}
#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "The test uses assertions while Result keeps setup failures readable."
)]
fn open_extracts_nested_primary_field_value() -> Result<()> {
    let result = open_arguments(Some(
        json ! ({ "requests" : [{ "url" : { "href" : "example.com" } , "chunk" : null }] }),
    ))?;
    let request = result
        .value
        .requests
        .first()
        .ok_or_else(|| crate::error::AppError::internal("missing normalized request"))?;
    assert_eq!(request.url, "https://example.com");
    assert_eq!(request.chunk, 0);
    Ok(())
}
#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "The test uses assertions while Result keeps setup failures readable."
)]
fn search_ignores_null_domain_items() -> Result<()> {
    let result = search_arguments(Some(
        json ! ({ "requests" : [{ "q" : "rust" , "domains" : ["example.com" , null , "rust-lang.org"] }] }),
    ))?;
    let request = result
        .value
        .requests
        .first()
        .ok_or_else(|| crate::error::AppError::internal("missing normalized request"))?;
    assert_eq!(
        request.domains.as_deref(),
        Some(&["example.com".to_owned(), "rust-lang.org".to_owned()][..])
    );
    Ok(())
}
#[test]
fn search_does_not_infer_query_when_named_field_exists() {
    let error = search_arguments(Some(json ! ({ "category" : "news" })))
        .unwrap_err()
        .client_message();
    assert!(error.contains("requests[0].q is required"));
}
#[test]
fn open_does_not_infer_url_when_named_field_exists() {
    let error = open_arguments(Some(json ! ({ "chunk" : "1" })))
        .unwrap_err()
        .client_message();
    assert!(error.contains("requests[0].url is required"));
}
