#![expect(
    clippy::nursery,
    clippy::pedantic,
    clippy::restriction,
    reason = "Dynamic JSON normalization needs generic value traversal."
)]
use crate::{
    Result,
    arguments::{
        aliases,
        support::{parse_json_container, push_unique, requests_entry, warn_unused_top_fields},
    },
    error::AppError,
};
use sonic_rs::{JsonContainerTrait, JsonValueTrait, Value};
use std::collections::BTreeMap;
pub type RequestMap = BTreeMap<String, Value>;
pub fn normalize_requests(
    raw: Option<Value>,
    fields: &[aliases::FieldSpec],
) -> Result<(Vec<RequestMap>, Vec<String>)> {
    let mut warnings = Vec::new();
    let Some(arguments) = raw.map(parse_json_container) else {
        return Err(AppError::client(
            "Invalid request: requests is required and must be an array",
        ));
    };
    if let Some(object) = arguments.as_object() {
        if let Some((key, value)) = requests_entry(object) {
            if key != "requests" {
                push_unique(
                    &mut warnings,
                    format!("use \"requests\" instead of \"{key}\""),
                );
            }
            warn_unused_top_fields(object, &key, &mut warnings);
            return requests_from_value(value.clone(), fields, &mut warnings);
        }
        let keys = object.iter().map(|(key, _)| key.to_owned());
        if aliases::looks_like_request(fields, keys) {
            push_unique(
                &mut warnings,
                "wrap the request object in the \"requests\" array".to_owned(),
            );
            let request = normalize_request_object(object, fields, &mut warnings, "request");
            return Ok((vec![request], warnings));
        }
        return Err(AppError::client(
            "Invalid request: requests is required and must be an array",
        ));
    }
    requests_from_value(arguments, fields, &mut warnings)
}
fn requests_from_value(
    value: Value,
    fields: &[aliases::FieldSpec],
    warnings: &mut Vec<String>,
) -> Result<(Vec<RequestMap>, Vec<String>)> {
    let parsed = parse_json_container(value);
    if let Some(array) = parsed.as_array() {
        let mut requests = Vec::with_capacity(array.len());
        for (index, item) in array.iter().enumerate() {
            requests.push(normalize_request_value(
                item.clone(),
                fields,
                warnings,
                &format!("requests[{index}]"),
            ));
        }
        return Ok((requests, warnings.clone()));
    }
    push_unique(warnings, "pass \"requests\" as an array".to_owned());
    Ok((
        vec![normalize_request_value(
            parsed,
            fields,
            warnings,
            "requests[0]",
        )],
        warnings.clone(),
    ))
}
fn normalize_request_value(
    value: Value,
    fields: &[aliases::FieldSpec],
    warnings: &mut Vec<String>,
    path: &str,
) -> RequestMap {
    let parsed = parse_json_container(value);
    let Some(object) = parsed.as_object() else {
        return RequestMap::new();
    };
    normalize_request_object(object, fields, warnings, path)
}
fn normalize_request_object(
    object: &sonic_rs::Object,
    fields: &[aliases::FieldSpec],
    warnings: &mut Vec<String>,
    path: &str,
) -> RequestMap {
    let mut normalized = RequestMap::new();
    for (raw_key, raw_item) in object {
        let Some(canonical) = aliases::canonical_field(fields, raw_key) else {
            push_unique(
                warnings,
                format!("ignored unrecognized field \"{}.{raw_key}\"", path),
            );
            continue;
        };
        if canonical != raw_key {
            push_unique(
                warnings,
                format!("use \"{canonical}\" instead of \"{raw_key}\""),
            );
        }
        if normalized.contains_key(canonical) {
            push_unique(
                warnings,
                format!(
                    "multiple aliases for \"{canonical}\" were provided; the last value was used"
                ),
            );
        }
        normalized.insert(
            canonical.to_owned(),
            normalize_field(
                canonical,
                raw_item.clone(),
                warnings,
                &format!("{path}.{canonical}"),
            ),
        );
    }
    normalized
}
fn normalize_field(name: &str, value: Value, warnings: &mut Vec<String>, path: &str) -> Value {
    let parsed = parse_json_container(value);
    if name == "url" {
        return normalize_url(parsed, warnings, path);
    }
    if name == "category" {
        return normalize_category(parsed, warnings, path);
    }
    if name == "domains" && !parsed.is_null() && parsed.as_array().is_none() {
        push_unique(warnings, format!("pass \"{path}\" as an array"));
    }
    if name == "q" {
        warn_site_query(&parsed, warnings, path);
    }
    parsed
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
