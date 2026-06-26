use crate::{
    Result,
    arguments::{
        aliases,
        request::{RequestMap, RequestNormalizer},
        support::{parse_json_container, push_unique, requests_entry, warn_unused_top_fields},
    },
    error::AppError,
};
use sonic_rs::{JsonContainerTrait as _, Value};
#[inline]
pub fn normalize_requests(
    raw: Option<Value>,
    fields: &'static [aliases::FieldSpec],
    primary_field: Option<&'static str>,
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
            let (requests, next_warnings) =
                requests_from_value(value, fields, primary_field, &mut warnings);
            return Ok((requests, next_warnings));
        }
        let normalizer = RequestNormalizer::new(fields, primary_field);
        let keys = object.iter().map(|(key, _)| key.to_owned());
        if aliases::looks_like_request(fields, keys) {
            push_unique(
                &mut warnings,
                "wrap the request object in the \"requests\" array".to_owned(),
            );
            let request = normalizer.normalize_object(&arguments, object, &mut warnings, "request");
            return Ok((vec![request], warnings));
        }
        if let Some(request) = normalizer.singleton_request(&arguments, &mut warnings, "request") {
            return Ok((vec![request], warnings));
        }
        return Err(AppError::client(
            "Invalid request: requests is required and must be an array",
        ));
    }
    let (requests, next_warnings) =
        requests_from_value(arguments, fields, primary_field, &mut warnings);
    Ok((requests, next_warnings))
}
fn requests_from_value(
    value: Value,
    fields: &'static [aliases::FieldSpec],
    primary_field: Option<&'static str>,
    warnings: &mut Vec<String>,
) -> (Vec<RequestMap>, Vec<String>) {
    let parsed = parse_json_container(value);
    let normalizer = RequestNormalizer::new(fields, primary_field);
    if let Some(array) = parsed.as_array() {
        let mut requests = Vec::with_capacity(array.len());
        for (index, item) in array.iter().enumerate() {
            requests.push(normalizer.normalize_value(
                item.clone(),
                warnings,
                &format!("requests[{index}]"),
            ));
        }
        return (requests, warnings.clone());
    }
    push_unique(warnings, "pass \"requests\" as an array".to_owned());
    (
        vec![normalizer.normalize_value(parsed, warnings, "requests[0]")],
        warnings.clone(),
    )
}
