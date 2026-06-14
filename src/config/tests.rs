use crate::{Result, config};
#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "The test uses assertions while Result keeps setup failures readable."
)]
fn embedded_config_is_valid() -> Result<()> {
    let loaded = config::load_embedded()?;
    loaded.validate()?;
    assert_eq!(loaded.server.name, "web");
    assert!(config::default_yaml().contains("server:"));
    Ok(())
}
#[test]
fn embedded_config_has_compile_time_source_name() {
    assert_eq!(config::embedded::source_name(), "config/default.yaml");
}
#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "The test uses assertions while Result keeps setup failures readable."
)]
fn embedded_config_keeps_runtime_free_defaults() -> Result<()> {
    let loaded = config::load_embedded()?;
    assert_eq!(loaded.server.streamable_http_path, "/mcp");
    assert!(!loaded.server.stateful_http);
    assert!(loaded.server.json_response);
    assert_eq!(loaded.search.endpoint, "https://api.exa.ai/search");
    Ok(())
}
