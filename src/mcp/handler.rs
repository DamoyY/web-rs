use crate::config::AppConfig;
use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
#[derive(Serialize)]
pub(crate) struct HealthPayload {
    status: &'static str,
    name: String,
}
#[inline]
pub(crate) async fn health(State(config): State<AppConfig>) -> (StatusCode, Json<HealthPayload>) {
    (
        StatusCode::OK,
        Json(HealthPayload {
            status: "ok",
            name: config.server.name,
        }),
    )
}
