use super::extract_markdowns;
use crate::Result;
#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "The test uses assertions while Result keeps setup failures readable."
)]
fn response_extracts_requested_result_text() -> Result<()> {
    let body = br##"{
        "results": [
            {
                "url": "https://example.com",
                "text": "# Example"
            }
        ],
        "errors": []
    }"##;
    let urls = vec!["https://example.com".to_owned()];
    assert_eq!(extract_markdowns(&urls, body)?, ["# Example"]);
    Ok(())
}
#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "The test uses assertions while Result keeps setup failures readable."
)]
fn response_extracts_batch_results_in_request_order() -> Result<()> {
    let body = br##"{
        "results": [
            {
                "url": "https://example.com/b",
                "text": "# B"
            },
            {
                "url": "https://example.com/a",
                "text": "# A"
            }
        ],
        "errors": []
    }"##;
    let urls = vec![
        "https://example.com/a".to_owned(),
        "https://example.com/b".to_owned(),
    ];
    assert_eq!(extract_markdowns(&urls, body)?, ["# A", "# B"]);
    Ok(())
}
#[test]
fn response_surfaces_per_url_error() {
    let body = br#"{
        "results": [],
        "errors": [
            {
                "url": "https://example.com",
                "error": "bot_blocked",
                "status": null
            }
        ]
    }"#;
    let urls = vec!["https://example.com".to_owned()];
    let error = extract_markdowns(&urls, body).unwrap_err().client_message();
    assert_eq!(
        error,
        "TinyFish could not fetch https://example.com: bot_blocked."
    );
}
