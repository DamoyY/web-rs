use crate::arguments::{
    aliases,
    singleton::single_valid_string,
    support::{parse_json_container, push_unique},
};
use sonic_rs::{JsonContainerTrait as _, JsonValueTrait as _, Value};
pub(crate) fn normalize_field(
    name: &str,
    value: Value,
    warnings: &mut Vec<String>,
    path: &str,
) -> Value {
    let parsed = parse_json_container(value);
    if name == "url" {
        return normalize_url(normalize_text(parsed), warnings, path);
    }
    if name == "category" {
        return normalize_category(parsed, warnings, path);
    }
    if name == "domains" && !parsed.is_null() && parsed.as_array().is_none() {
        push_unique(warnings, format!("pass \"{path}\" as an array"));
    }
    if name == "q" {
        let query = normalize_text(parsed);
        warn_site_query(&query, warnings, path);
        return query;
    }
    parsed
}
fn normalize_text(value: Value) -> Value {
    if value.as_str().is_some() {
        return value;
    }
    let Some(text) = single_valid_string(&value) else {
        return value;
    };
    Value::from(text.as_str())
}
fn normalize_url(value: Value, warnings: &mut Vec<String>, path: &str) -> Value {
    let Some(text) = value.as_str() else {
        return value;
    };
    let trimmed = text.trim();
    if trimmed != text {
        push_unique(
            warnings,
            format!("remove surrounding whitespace from \"{path}\""),
        );
    }
    if trimmed.is_empty() || trimmed.contains("://") {
        return Value::from(trimmed);
    }
    let normalized = format!("https://{trimmed}");
    push_unique(
        warnings,
        format!("include a URL scheme for \"{path}\"; interpreted as \"{normalized}\""),
    );
    Value::from(normalized.as_str())
}
fn normalize_category(value: Value, warnings: &mut Vec<String>, path: &str) -> Value {
    let Some(text) = value.as_str() else {
        return value;
    };
    let Some(category) = aliases::canonical_category(text) else {
        return value;
    };
    if category != text {
        push_unique(
            warnings,
            format!("use \"{category}\" instead of \"{text}\" for \"{path}\""),
        );
    }
    Value::from(category)
}
fn warn_site_query(value: &Value, warnings: &mut Vec<String>, path: &str) {
    let Some(text) = value.as_str() else {
        return;
    };
    if text.to_ascii_lowercase().contains("site:") {
        push_unique(
            warnings,
            format!("use \"domains\" instead of site: syntax in \"{path}\""),
        );
    }
}
