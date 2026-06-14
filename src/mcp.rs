pub mod handler;
pub mod protocol;
pub mod schemas;
pub mod tools;
use crate::{Result, config::AppConfig};
pub(crate) fn state(config: AppConfig) -> Result<handler::AppState> {
    Ok(handler::AppState {
        tools: tools::ToolService::new(config.clone())?,
        config,
    })
}
#[must_use]
pub(crate) const fn protocol_content_type() -> &'static str {
    "application/json; charset=utf-8"
}
