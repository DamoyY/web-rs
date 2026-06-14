use super::{extract_content, extract_payload_content};
use crate::Result;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use sonic_rs::Value;
#[test]
fn payload_content_reads_plain_object_text() {
    let payload = sonic_rs :: json ! ({ "content" : "alpha beta" , });
    assert_eq!(
        extract_payload_content(&payload),
        Some("alpha beta".to_owned())
    );
}
#[test]
fn payload_content_reads_nested_data_text() {
    let payload = sonic_rs :: json ! ({ "data" : { "markdown" : "nested markdown" , } , });
    assert_eq!(
        extract_payload_content(&payload),
        Some("nested markdown".to_owned())
    );
}
#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "The test uses assertions while Result keeps setup failures readable."
)]
fn json_content_extracts_plain_object_text() -> Result<()> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    let payload: Value = sonic_rs :: json ! ({ "text" : "plain json text" , });
    let body = sonic_rs::to_vec(&payload).map_err(|error| {
        crate::error::AppError::internal(format!("failed to encode test payload: {error}"))
    })?;
    assert_eq!(extract_content(&headers, &body)?, "plain json text");
    Ok(())
}
