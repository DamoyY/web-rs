use sonic_rs::{JsonValueTrait, Object, Value};
#[must_use]
pub(crate) fn parse_json_container(value: Value) -> Value {
    let Some(text) = value.as_str() else {
        return value;
    };
    let trimmed = text.trim();
    if !trimmed.starts_with(['{', '[']) {
        return value;
    }
    sonic_rs::from_str(trimmed).unwrap_or(value)
}
pub(crate) fn push_unique(warnings: &mut Vec<String>, message: String) {
    if !warnings.iter().any(|existing| existing == &message) {
        warnings.push(message);
    }
}
pub(crate) fn requests_entry(object: &Object) -> Option<(String, Value)> {
    object.iter().find_map(|(key, value)| {
        (key.eq_ignore_ascii_case("requests") || key.eq_ignore_ascii_case("request"))
            .then(|| (key.to_owned(), value.clone()))
    })
}
pub(crate) fn warn_unused_top_fields(object: &Object, used: &str, warnings: &mut Vec<String>) {
    for (key, _) in object {
        if key == used {
            continue;
        }
        push_unique(warnings, format!("ignored unrecognized field \"{key}\""));
    }
}
