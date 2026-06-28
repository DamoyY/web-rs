use super::append_chunk;
#[test]
fn limited_append_allows_exact_limit() {
    let mut body = b"abc".to_vec();
    append_chunk(&mut body, b"de", 5).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(body, b"abcde");
}
#[test]
fn limited_append_rejects_oversize_chunk_without_extending_body() {
    let mut body = b"abc".to_vec();
    let error = append_chunk(&mut body, b"def", 5)
        .unwrap_err()
        .client_message();
    assert_eq!(body, b"abc");
    assert_eq!(
        error,
        "HTTP response body exceeded the configured 5 byte limit."
    );
}
