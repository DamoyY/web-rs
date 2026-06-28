use super::Cli;
use clap::Parser as _;
#[test]
fn exa_api_key_configures_stdio_credentials() {
    let cli = Cli::try_parse_from(["web-rs", "--exa-api-key", "exa"])
        .unwrap_or_else(|error| panic!("{error}"));
    let options = cli
        .runtime_options()
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(options.credentials.exa_api_key.as_deref(), Some("exa"));
}
#[test]
fn rejects_multiple_reader_api_keys() {
    let cli = Cli::try_parse_from([
        "web-rs",
        "--jina-api-key",
        "jina",
        "--tinyfish-api-key",
        "tinyfish",
    ])
    .unwrap_or_else(|error| panic!("{error}"));
    let error = cli.runtime_options().unwrap_err().client_message();
    assert!(error.contains("at most one"));
}
