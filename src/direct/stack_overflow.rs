use crate::{Result, error::AppError};
use serde::Serialize;
use sonic_rs::{JsonContainerTrait as _, JsonValueTrait as _, Object, Value};
use url::Url;
const API_FILTER: &str = "W-vZ8WEHVi3D2JhQe1m8l90EjOxo6eCsb6b_6yfX0_p";
const MAX_ANSWERS_PER_REQUEST: &str = "100";
#[derive(Serialize)]
struct QuestionAndAnswers {
    question: Object,
    answers: Vec<Object>,
}
#[must_use]
#[inline]
pub fn resolve_stack_overflow_api_url(parsed: &Url) -> Option<String> {
    let host = parsed.host_str()?.to_ascii_lowercase();
    if !matches!(host.as_str(), "stackoverflow.com" | "www.stackoverflow.com") {
        return None;
    }
    let question_id = question_id(parsed)?;
    let mut api = Url::parse(&format!(
        "https://api.stackexchange.com/2.3/questions/{question_id}"
    ))
    .ok()?;
    api.query_pairs_mut()
        .append_pair("order", "desc")
        .append_pair("sort", "votes")
        .append_pair("site", "stackoverflow")
        .append_pair("page", "1")
        .append_pair("pagesize", MAX_ANSWERS_PER_REQUEST)
        .append_pair("filter", API_FILTER);
    Some(api.to_string())
}
#[inline]
pub fn format_stack_overflow_question_json(payload: &Value) -> Result<String> {
    if let Some(message) = api_error_message(payload) {
        return Err(AppError::client(message));
    }
    if payload
        .get("has_more")
        .and_then(Value::as_bool)
        .unwrap_or_default()
    {
        return Err(AppError::client(
            "Stack Exchange API returned more than 100 answers; direct fetch cannot return a complete answer list.",
        ));
    }
    let item = single_question_item(payload)?;
    let question_object = item.as_object().ok_or_else(|| {
        AppError::client("Stack Exchange API returned an invalid question object.")
    })?;
    let output = QuestionAndAnswers {
        question: stripped_object(question_object, &["answers", "comments", "comment_count"]),
        answers: answer_objects(question_object)?,
    };
    sonic_rs::to_string_pretty(&output).map_err(|error| {
        AppError::internal(format!("failed to serialize Stack Overflow JSON: {error}"))
    })
}
fn question_id(parsed: &Url) -> Option<u64> {
    let parts: Vec<&str> = parsed
        .path_segments()?
        .filter(|part| !part.is_empty())
        .collect();
    let first = parts.first()?;
    let second = parts.get(1)?;
    if matches!(*first, "questions" | "q") {
        return parse_id(second);
    }
    None
}
fn parse_id(value: &str) -> Option<u64> {
    let parsed = value.parse::<u64>().ok()?;
    (parsed > 0).then_some(parsed)
}
fn single_question_item(payload: &Value) -> Result<&Value> {
    let items = payload
        .get("items")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::client("Stack Exchange API response is missing questions."))?;
    if items.len() != 1 {
        return Err(AppError::client(
            "Stack Exchange API did not return exactly one question.",
        ));
    }
    items
        .as_slice()
        .first()
        .ok_or_else(|| AppError::client("Stack Exchange API did not return exactly one question."))
}
fn answer_objects(question: &Object) -> Result<Vec<Object>> {
    let Some(answers) = question.get(&"answers") else {
        return Ok(Vec::new());
    };
    let array = answers
        .as_array()
        .ok_or_else(|| AppError::client("Stack Exchange API returned invalid answers."))?;
    array
        .iter()
        .map(|answer| {
            answer
                .as_object()
                .map(|object| stripped_object(object, &["comments", "comment_count"]))
                .ok_or_else(|| AppError::client("Stack Exchange API returned an invalid answer."))
        })
        .collect()
}
fn stripped_object(object: &Object, excluded: &[&str]) -> Object {
    let mut stripped = Object::with_capacity(object.len());
    for (key, value) in object {
        if !excluded.contains(&key) {
            stripped.insert(key, value.clone());
        }
    }
    stripped
}
fn api_error_message(payload: &Value) -> Option<String> {
    let error_id = payload.get("error_id").and_then(Value::as_i64)?;
    let error_name = payload
        .get("error_name")
        .and_then(Value::as_str)
        .unwrap_or("unknown_error");
    let error_message = payload
        .get("error_message")
        .and_then(Value::as_str)
        .unwrap_or("Stack Exchange API rejected the question request.");
    Some(format!(
        "Stack Exchange API rejected the question request ({error_name}/{error_id}): {error_message}"
    ))
}
