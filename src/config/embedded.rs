use crate::{Result, config::AppConfig, error::AppError};
pub const DEFAULT_YAML: &str = include_str!("../../config/default.yaml");
#[must_use]
pub const fn source_name() -> &'static str {
    "config/default.yaml"
}
pub fn load() -> Result<AppConfig> {
    load_from_str(DEFAULT_YAML)
}
pub(crate) fn load_from_str(yaml: &str) -> Result<AppConfig> {
    serde_saphyr::from_str(yaml).map_err(|error| {
        AppError::config(format!(
            "embedded YAML {} is invalid: {error}",
            source_name()
        ))
    })
}
