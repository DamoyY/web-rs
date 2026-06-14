use serde::{Deserialize, Serialize};
use sonic_rs::Value;
#[derive(Debug, Deserialize)]
pub struct RpcRequest {
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}
#[derive(Serialize)]
pub struct RpcSuccess<T> {
    pub jsonrpc: &'static str,
    pub id: Value,
    pub result: T,
}
#[derive(Serialize)]
pub struct RpcFailure {
    pub jsonrpc: &'static str,
    pub id: Option<Value>,
    pub error: RpcError,
}
#[derive(Serialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}
#[derive(Deserialize)]
pub struct CallToolParams {
    pub name: String,
    #[serde(default)]
    pub arguments: Option<Value>,
}
#[derive(Serialize)]
pub struct InitializeResult<'config> {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: &'config str,
    pub capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo<'config>,
    pub instructions: &'config str,
}
#[derive(Serialize)]
pub struct ServerCapabilities {
    pub tools: ToolCapabilities,
}
#[derive(Serialize)]
pub struct ToolCapabilities {
    #[serde(rename = "listChanged")]
    pub list_changed: bool,
}
#[derive(Serialize)]
pub struct ServerInfo<'config> {
    pub name: &'config str,
    pub version: &'config str,
}
#[derive(Serialize)]
pub struct ToolCallResult {
    pub content: Vec<TextContent>,
    #[serde(rename = "structuredContent")]
    pub structured_content: Value,
}
#[derive(Serialize)]
pub struct TextContent {
    #[serde(rename = "type")]
    pub content_type: &'static str,
    pub text: String,
}
