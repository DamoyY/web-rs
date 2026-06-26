use super::extract_markdown;
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
    assert_eq!(extract_markdown("https://example.com", body)?, "# Example");
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
    let error = extract_markdown("https://example.com", body)
        .unwrap_err()
        .client_message();
    assert_eq!(
        error,
        "TinyFish could not fetch https://example.com: bot_blocked."
    );
}
