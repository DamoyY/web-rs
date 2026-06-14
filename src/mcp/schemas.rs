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
use schemars::{JsonSchema, schema_for};
use serde::Serialize;
use sonic_rs::Value;
#[derive(Serialize)]
pub struct ToolList {
    pub tools: Vec<ToolDescription>,
}
#[derive(Serialize)]
pub struct ToolDescription {
    pub name: &'static str,
    pub description: &'static str,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}
pub fn tool_list() -> Result<ToolList> {
    Ok(ToolList {
        tools: vec![
            ToolDescription {
                name: "search_query",
                description: "返回标题、日期、URL 与摘要。",
                input_schema: schema_value::<SearchQueryArguments>()?,
            },
            ToolDescription {
                name: "open",
                description: "用于读取页面内容。",
                input_schema: schema_value::<OpenArguments>()?,
            },
            ToolDescription {
                name: "find",
                description: "在页面中使用正则表达式查找匹配片段。",
                input_schema: schema_value::<FindArguments>()?,
            },
        ],
    })
}
fn schema_value<T>() -> Result<Value>
where
    T: JsonSchema,
{
    sonic_rs::to_value(&schema_for!(T))
        .map_err(|error| AppError::internal(format!("failed to build tool schema: {error}")))
}
