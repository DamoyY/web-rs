#![expect(
    clippy::map_err_ignore,
    clippy::missing_inline_in_public_items,
    clippy::shadow_unrelated,
    clippy::unused_trait_names,
    reason = "MCP JSON-RPC handlers preserve protocol field names."
)]
use crate::{
    VERSION,
    config::AppConfig,
    error::AppError,
    mcp::{
        protocol::{
            CallToolParams, InitializeResult, RpcError, RpcFailure, RpcRequest, RpcSuccess,
            ServerCapabilities, ServerInfo, ToolCallResult, ToolCapabilities,
        },
        protocol_content_type, schemas,
        tools::ToolService,
    },
};
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
};
use sonic_rs::{Value, json};
#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub tools: ToolService,
}
pub async fn health(State(state): State<AppState>) -> Response {
    json_response(
        StatusCode::OK,
        &json ! ({ "status" : "ok" , "name" : state . config . server . name }),
    )
}
pub async fn mcp_entrypoint(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let request = match sonic_rs::from_slice::<RpcRequest>(&body) {
        Ok(value) => value,
        Err(error) => return rpc_error(None, -32700, format!("Parse error: {error}")),
    };
    let Some(id) = request.id.clone() else {
        return StatusCode::ACCEPTED.into_response();
    };
    match dispatch(&state, &headers, request).await {
        Ok(result) => rpc_success(id, result),
        Err(error) => rpc_error(Some(id), -32000, error.client_message()),
    }
}
async fn dispatch(
    state: &AppState,
    headers: &HeaderMap,
    request: RpcRequest,
) -> crate::Result<Value> {
    match request.method.as_str() {
        "initialize" => sonic_rs::to_value(&initialize_result(&state.config)).map_err(|error| {
            AppError::internal(format!("failed to encode initialize result: {error}"))
        }),
        "ping" => Ok(json!({})),
        "tools/list" => sonic_rs::to_value(&schemas::tool_list()?)
            .map_err(|error| AppError::internal(format!("failed to encode tool list: {error}"))),
        "tools/call" => call_tool(state, headers, request.params).await,
        method => Err(AppError::client(format!(
            "Unsupported MCP method: {method}"
        ))),
    }
}
async fn call_tool(
    state: &AppState,
    headers: &HeaderMap,
    params: Option<Value>,
) -> crate::Result<Value> {
    let Some(raw_params) = params else {
        return Err(AppError::client("tools/call params are required."));
    };
    let params: CallToolParams = sonic_rs::from_value(&raw_params)
        .map_err(|_| AppError::client("tools/call params are invalid."))?;
    let structured = state
        .tools
        .call(&params.name, params.arguments, headers)
        .await?;
    let text = sonic_rs::to_string_pretty(&structured)
        .map_err(|error| AppError::internal(format!("failed to render tool result: {error}")))?;
    sonic_rs::to_value(&ToolCallResult {
        content: vec![crate::mcp::protocol::TextContent {
            content_type: "text",
            text,
        }],
        structured_content: structured,
    })
    .map_err(|error| AppError::internal(format!("failed to encode tool call result: {error}")))
}
fn initialize_result(config: &AppConfig) -> InitializeResult<'_> {
    InitializeResult {
        protocol_version: &config.server.protocol_version,
        capabilities: ServerCapabilities {
            tools: ToolCapabilities {
                list_changed: false,
            },
        },
        server_info: ServerInfo {
            name: &config.server.name,
            version: VERSION,
        },
        instructions: &config.server.instructions,
    }
}
fn rpc_success(id: Value, result: Value) -> Response {
    json_response(
        StatusCode::OK,
        &RpcSuccess {
            jsonrpc: "2.0",
            id,
            result,
        },
    )
}
fn rpc_error(id: Option<Value>, code: i32, message: String) -> Response {
    json_response(
        StatusCode::OK,
        &RpcFailure {
            jsonrpc: "2.0",
            id,
            error: RpcError { code, message },
        },
    )
}
fn json_response<T>(status: StatusCode, value: &T) -> Response
where
    T: serde::Serialize,
{
    match sonic_rs::to_vec(value) {
        Ok(body) => (status, [(CONTENT_TYPE, protocol_content_type())], body).into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(CONTENT_TYPE, "text/plain; charset=utf-8")],
            format!("JSON serialization failed: {error}"),
        )
            .into_response(),
    }
}
