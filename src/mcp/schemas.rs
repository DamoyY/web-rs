#![expect(
    clippy::pedantic,
    clippy::restriction,
    reason = "Tool schema names intentionally match MCP tool names."
)]
use crate::{
    Result,
    error::AppError,
    models::{FindArguments, OpenArguments, SearchQueryArguments},
};
use rmcp::model::{JsonObject, Tool};
use schemars::{JsonSchema, schema_for};
pub fn tools() -> Result<Vec<Tool>> {
    Ok(vec![
        tool::<SearchQueryArguments>("search_query", "返回标题、日期、URL 与摘要。")?,
        tool::<OpenArguments>("open", "用于读取页面内容。")?,
        tool::<FindArguments>("find", "在页面中使用正则表达式查找匹配片段。")?,
    ])
}
pub fn tool_by_name(name: &str) -> Result<Option<Tool>> {
    Ok(tools()?.into_iter().find(|tool| tool.name == name))
}
fn tool<T>(name: &'static str, description: &'static str) -> Result<Tool>
where
    T: JsonSchema,
{
    Ok(Tool::new(name, description, schema_object::<T>()?))
}
fn schema_object<T>() -> Result<JsonObject>
where
    T: JsonSchema,
{
    let value = rmcp::serde_json::to_value(schema_for!(T))
        .map_err(|error| AppError::internal(format!("failed to build tool schema: {error}")))?;
    match value {
        rmcp::serde_json::Value::Object(object) => Ok(object),
        _ => Err(AppError::internal("tool schema is not a JSON object")),
    }
}
