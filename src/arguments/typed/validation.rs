use crate::{Result, arguments::request::RequestMap, error::AppError, models::SearchCategory};
use sonic_rs::{JsonContainerTrait as _, JsonValueTrait as _, Value};
use url::Url;
pub(super) fn required_http_url(map: &RequestMap, field: &str, path: &str) -> Result<String> {
    let value = required_string(map, field, path)?;
    let parsed = Url::parse(&value)
        .map_err(|_error| validation(format!("{path} must be an absolute HTTP or HTTPS URL")))?;
    if matches!(parsed.scheme(), "http" | "https") && parsed.host_str().is_some() {
        return Ok(value);
    }
    Err(validation(format!(
        "{path} must be an absolute HTTP or HTTPS URL"
    )))
}
pub(super) fn required_string(map: &RequestMap, field: &str, path: &str) -> Result<String> {
    let value = map
        .get(field)
        .ok_or_else(|| validation(format!("{path} is required")))?;
    let Some(text) = value.as_str() else {
        return Err(validation(format!("{path} must be a string")));
    };
    if text.is_empty() {
        return Err(validation(format!("{path} must not be empty")));
    }
    Ok(text.to_owned())
}
pub(super) fn required_usize(map: &RequestMap, field: &str, path: &str) -> Result<usize> {
    let value = map
        .get(field)
        .ok_or_else(|| validation(format!("{path} is required")))?;
    non_negative_usize(value, path)
}
pub(super) fn optional_usize(map: &RequestMap, field: &str, path: &str) -> Result<Option<usize>> {
    let Some(value) = map.get(field) else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    positive_usize(value, path).map(Some)
}
pub(super) fn optional_string_list(
    map: &RequestMap,
    field: &str,
    path: &str,
) -> Result<Option<Vec<String>>> {
    let Some(value) = map.get(field) else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    if let Some(array) = value.as_array() {
        let mut items = Vec::with_capacity(array.len());
        for (index, item) in array.iter().enumerate() {
            if item.is_null() {
                continue;
            }
            items.push(value_string(item, &format!("{path}[{index}]"))?);
        }
        return Ok(Some(items));
    }
    value_string(value, path).map(|item| Some(vec![item]))
}
pub(super) fn optional_category(map: &RequestMap, path: &str) -> Result<Option<SearchCategory>> {
    let Some(value) = map.get("category") else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let text = value_string(value, path)?;
    match text.as_str() {
        "company" => Ok(Some(SearchCategory::Company)),
        "research paper" => Ok(Some(SearchCategory::ResearchPaper)),
        "news" => Ok(Some(SearchCategory::News)),
        "pdf" => Ok(Some(SearchCategory::Pdf)),
        "personal site" => Ok(Some(SearchCategory::PersonalSite)),
        "financial report" => Ok(Some(SearchCategory::FinancialReport)),
        "people" => Ok(Some(SearchCategory::People)),
        _ => Err(validation(format!(
            "{path} must be one of the documented values"
        ))),
    }
}
fn positive_usize(value: &Value, path: &str) -> Result<usize> {
    let Some(number) = integer_value(value) else {
        return Err(validation(format!("{path} must be an integer")));
    };
    if number < 1 {
        return Err(validation(format!(
            "{path} must be greater than or equal to 1"
        )));
    }
    usize::try_from(number).map_err(|_overflow| validation(format!("{path} is too large")))
}
fn non_negative_usize(value: &Value, path: &str) -> Result<usize> {
    let Some(number) = integer_value(value) else {
        return Err(validation(format!("{path} must be an integer")));
    };
    if number < 0 {
        return Err(validation(format!(
            "{path} must be greater than or equal to 0"
        )));
    }
    usize::try_from(number).map_err(|_overflow| validation(format!("{path} is too large")))
}
pub(super) fn integer_value(value: &Value) -> Option<i128> {
    if value.as_bool().is_some() {
        return None;
    }
    if let Some(number) = value.as_i64() {
        return Some(i128::from(number));
    }
    if let Some(number) = value.as_u64() {
        return Some(i128::from(number));
    }
    value.as_str()?.trim().parse::<i128>().ok()
}
fn value_string(value: &Value, path: &str) -> Result<String> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| validation(format!("{path} must be a string")))
}
pub(super) fn is_empty(value: &Value) -> bool {
    if value.is_null() {
        return true;
    }
    if let Some(text) = value.as_str() {
        return text.trim().is_empty();
    }
    value.as_array().is_some_and(sonic_rs::Array::is_empty)
        || value.as_object().is_some_and(sonic_rs::Object::is_empty)
}
#[expect(
    clippy::needless_pass_by_value,
    reason = "Validation callers build owned messages that are immediately prefixed."
)]
fn validation(message: String) -> AppError {
    AppError::client(format!("Invalid request: {message}"))
}
