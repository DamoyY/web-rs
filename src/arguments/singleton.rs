use crate::arguments::support::parse_json_container;
use sonic_rs::{JsonContainerTrait as _, JsonValueTrait as _, Value};
#[must_use]
pub(crate) fn single_valid_string(value: &Value) -> Option<String> {
    let mut finder = StringFinder::default();
    finder.visit(value);
    if finder.multiple {
        return None;
    }
    finder.value
}
#[derive(Default)]
struct StringFinder {
    value: Option<String>,
    multiple: bool,
}
impl StringFinder {
    fn visit(&mut self, value: &Value) {
        if self.multiple {
            return;
        }
        let parsed = parse_json_container(value.clone());
        if let Some(text) = parsed.as_str() {
            self.push(text);
            return;
        }
        if let Some(array) = parsed.as_array() {
            for item in array {
                self.visit(item);
            }
            return;
        }
        if let Some(object) = parsed.as_object() {
            for (_key, item) in object {
                self.visit(item);
            }
        }
    }
    fn push(&mut self, text: &str) {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return;
        }
        if self.value.is_some() {
            self.multiple = true;
            return;
        }
        self.value = Some(trimmed.to_owned());
    }
}
