use crate::{
    Result,
    arguments::{
        Normalized,
        aliases::{FIND_FIELDS, OPEN_FIELDS, SEARCH_FIELDS},
        normalizer::{RequestMap, normalize_requests},
        support::push_unique,
    },
    error::AppError,
    models::{
        FindArguments, FindRequest, OpenArguments, OpenRequest, SearchCategory,
        SearchQueryArguments, SearchQueryRequest,
    },
};
use sonic_rs::{JsonContainerTrait as _, JsonValueTrait as _, Value};
use url::Url;
#[inline]
pub fn search_arguments(raw: Option<Value>) -> Result<Normalized<SearchQueryArguments>> {
    let (maps, warnings) = normalize_requests(raw, SEARCH_FIELDS)?;
    let mut requests = Vec::with_capacity(maps.len());
    for (index, map) in maps.iter().enumerate() {
        requests.push(SearchQueryRequest {
            q: required_string(map, "q", &format!("requests[{index}].q"))?,
            recency: optional_usize(map, "recency", &format!("requests[{index}].recency"))?
                .map(u64::try_from)
                .transpose()
                .map_err(|_overflow| {
                    AppError::client("Invalid request: requests[].recency is too large")
                })?,
            domains: optional_string_list(map, "domains", &format!("requests[{index}].domains"))?,
            category: optional_category(map, &format!("requests[{index}].category"))?,
        });
    }
    Ok(Normalized::new(SearchQueryArguments { requests }, warnings))
}
#[inline]
pub fn open_arguments(raw: Option<Value>) -> Result<Normalized<OpenArguments>> {
    let (mut maps, mut warnings) = normalize_requests(raw, OPEN_FIELDS)?;
    normalize_open_chunks(&mut maps, &mut warnings);
    let mut requests = Vec::with_capacity(maps.len());
    for (index, map) in maps.iter().enumerate() {
        let url = required_http_url(map, "url", &format!("requests[{index}].url"))?;
        let chunk = required_usize(map, "chunk", &format!("requests[{index}].chunk"))?;
        requests.push(OpenRequest { url, chunk });
    }
    Ok(Normalized::new(OpenArguments { requests }, warnings))
}
#[inline]
pub fn find_arguments(raw: Option<Value>) -> Result<Normalized<FindArguments>> {
    let (maps, warnings) = normalize_requests(raw, FIND_FIELDS)?;
    let mut requests = Vec::with_capacity(maps.len());
    for (index, map) in maps.iter().enumerate() {
        requests.push(FindRequest {
            url: required_http_url(map, "url", &format!("requests[{index}].url"))?,
            pattern: required_string(map, "pattern", &format!("requests[{index}].pattern"))?,
            snippet_tokens: optional_usize(
                map,
                "snippet_tokens",
                &format!("requests[{index}].snippet_tokens"),
            )?,
        });
    }
    Ok(Normalized::new(FindArguments { requests }, warnings))
}
fn normalize_open_chunks(maps: &mut [RequestMap], warnings: &mut Vec<String>) {
    for (index, map) in maps.iter_mut().enumerate() {
        let path = format!("requests[{index}].chunk");
        match map.get("chunk") {
            None => {
                map.insert("chunk".to_owned(), Value::from(1_u64));
                push_unique(warnings, format!("\"{path}\" is required; using 1"));
            }
            Some(value) if is_empty(value) => {
                map.insert("chunk".to_owned(), Value::from(1_u64));
                push_unique(warnings, format!("\"{path}\" is empty; using 1"));
            }
            Some(value) if integer_value(value).is_some_and(|number| number < 1) => {
                map.insert("chunk".to_owned(), Value::from(1_u64));
                push_unique(
                    warnings,
                    format!("\"{path}\" must be greater than or equal to 1; using 1"),
                );
            }
            Some(_) => {}
        }
    }
}
fn required_http_url(map: &RequestMap, field: &str, path: &str) -> Result<String> {
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
fn required_string(map: &RequestMap, field: &str, path: &str) -> Result<String> {
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
fn required_usize(map: &RequestMap, field: &str, path: &str) -> Result<usize> {
    let value = map
        .get(field)
        .ok_or_else(|| validation(format!("{path} is required")))?;
    positive_usize(value, path)
}
fn optional_usize(map: &RequestMap, field: &str, path: &str) -> Result<Option<usize>> {
    let Some(value) = map.get(field) else {
        return Ok(None);
    };
    positive_usize(value, path).map(Some)
}
fn optional_string_list(map: &RequestMap, field: &str, path: &str) -> Result<Option<Vec<String>>> {
    let Some(value) = map.get(field) else {
        return Ok(None);
    };
    if let Some(array) = value.as_array() {
        return array
            .iter()
            .enumerate()
            .map(|(index, item)| value_string(item, &format!("{path}[{index}]")))
            .collect::<Result<Vec<_>>>()
            .map(Some);
    }
    value_string(value, path).map(|item| Some(vec![item]))
}
fn optional_category(map: &RequestMap, path: &str) -> Result<Option<SearchCategory>> {
    let Some(value) = map.get("category") else {
        return Ok(None);
    };
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
fn integer_value(value: &Value) -> Option<i128> {
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
fn is_empty(value: &Value) -> bool {
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
