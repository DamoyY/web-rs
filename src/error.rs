#![expect(
    clippy::exhaustive_enums,
    clippy::impl_trait_in_params,
    clippy::missing_inline_in_public_items,
    clippy::module_name_repetitions,
    clippy::pattern_type_mismatch,
    reason = "Error constructors accept caller-owned or borrowed messages."
)]
use thiserror::Error;
#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Client(String),
    #[error("configuration error: {0}")]
    Config(String),
    #[error("{0}")]
    Upstream(String),
    #[error("internal server error: {0}")]
    Internal(String),
}
impl AppError {
    #[must_use]
    pub fn client(message: impl Into<String>) -> Self {
        Self::Client(message.into())
    }
    #[must_use]
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }
    #[must_use]
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }
    #[must_use]
    pub fn client_message(&self) -> String {
        match self {
            Self::Client(message) | Self::Upstream(message) | Self::Config(message) => {
                message.clone()
            }
            Self::Internal(_) => {
                "Unexpected server error. Retry the request or contact the service operator."
                    .to_owned()
            }
        }
    }
}
impl From<reqwest::Error> for AppError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            return Self::Upstream("upstream request timed out. Retry later.".to_owned());
        }
        Self::Upstream(
            "Could not reach upstream service. Check network connectivity and retry.".to_owned(),
        )
    }
}
#[must_use]
pub fn http_service_error(service: &str, status_code: u16) -> AppError {
    if matches!(status_code, 401 | 403) {
        return AppError::client(format!(
            "{service} request was rejected. Check the API key header and permissions."
        ));
    }
    if status_code == 429 {
        return AppError::client(format!(
            "{service} rate limit was reached. Retry later or use another API key."
        ));
    }
    if (400..500).contains(&status_code) {
        return AppError::client(format!(
            "{service} rejected the request with HTTP {status_code}. Check the input URL and parameters."
        ));
    }
    AppError::client(format!(
        "{service} returned HTTP {status_code}. Retry later or contact the upstream service."
    ))
}
