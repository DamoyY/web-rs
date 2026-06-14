#![expect(
    clippy::pedantic,
    clippy::restriction,
    reason = "Public argument helpers keep MCP tool names explicit."
)]
pub mod aliases;
pub mod normalizer;
pub(crate) mod support;
#[cfg(test)]
mod tests;
pub mod typed;
use crate::{
    Result,
    models::{FindArguments, OpenArguments, SearchQueryArguments},
};
use sonic_rs::Value;
#[derive(Clone, Debug)]
pub struct Normalized<T> {
    pub value: T,
    pub warning: Option<Vec<String>>,
}
impl<T> Normalized<T> {
    #[must_use]
    pub fn new(value: T, warnings: Vec<String>) -> Self {
        Self {
            value,
            warning: (!warnings.is_empty()).then_some(warnings),
        }
    }
}
pub fn search_arguments(raw: Option<Value>) -> Result<Normalized<SearchQueryArguments>> {
    typed::search_arguments(raw)
}
pub fn open_arguments(raw: Option<Value>) -> Result<Normalized<OpenArguments>> {
    typed::open_arguments(raw)
}
pub fn find_arguments(raw: Option<Value>) -> Result<Normalized<FindArguments>> {
    typed::find_arguments(raw)
}
