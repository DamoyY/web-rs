#![expect(
    clippy::pedantic,
    clippy::restriction,
    reason = "Package registry URL rules require exact path segment handling."
)]
use percent_encoding::percent_decode_str;
use sonic_rs::JsonContainerTrait;
use url::Url;
#[derive(Debug)]
pub struct PackageRegistryTarget {
    pub request_url: String,
    pub json_fields_last: Vec<String>,
}
#[must_use]
pub fn resolve_package_registry_target(parsed: &Url) -> Option<PackageRegistryTarget> {
    let host = parsed.host_str()?.to_ascii_lowercase();
    let parts = path_parts(parsed);
    if host == "pypi.org" {
        return pypi_name(&parts).map(|name| PackageRegistryTarget {
            request_url: format!("https://pypi.org/pypi/{}/json", urlencoding::encode(&name)),
            json_fields_last: vec!["releases".to_owned()],
        });
    }
    if matches!(
        host.as_str(),
        "npmjs.com" | "www.npmjs.com" | "registry.npmjs.org"
    ) {
        return npm_name(&host, &parts).map(|name| PackageRegistryTarget {
            request_url: format!("https://registry.npmjs.org/{}", npm_encode(&name)),
            json_fields_last: vec!["versions".to_owned()],
        });
    }
    if host == "crates.io" {
        return crates_name(&parts).map(|name| PackageRegistryTarget {
            request_url: format!(
                "https://crates.io/api/v1/crates/{}",
                urlencoding::encode(&name)
            ),
            json_fields_last: vec!["versions".to_owned()],
        });
    }
    None
}
pub fn format_package_registry_json(
    payload: &sonic_rs::Value,
    fields_last: &[String],
) -> crate::Result<String> {
    if let Some(object) = payload.as_object() {
        let mut reordered = sonic_rs::Object::new();
        for (key, value) in object {
            if !fields_last.iter().any(|field| field == key) {
                reordered.insert(key, value.clone());
            }
        }
        for field in fields_last {
            if let Some(value) = object.get(field) {
                reordered.insert(field, value.clone());
            }
        }
        return sonic_rs::to_string_pretty(&reordered).map_err(|error| {
            crate::error::AppError::internal(format!("failed to serialize registry JSON: {error}"))
        });
    }
    sonic_rs::to_string_pretty(payload).map_err(|error| {
        crate::error::AppError::internal(format!("failed to serialize registry JSON: {error}"))
    })
}
fn pypi_name(parts: &[String]) -> Option<String> {
    if parts.len() >= 2 && parts[0] == "project" {
        return unquoted_segment(&parts[1]);
    }
    if parts.len() == 3 && parts[0] == "pypi" && parts[2] == "json" {
        return unquoted_segment(&parts[1]);
    }
    None
}
fn npm_name(host: &str, parts: &[String]) -> Option<String> {
    let package_parts = if matches!(host, "npmjs.com" | "www.npmjs.com") {
        (parts.first()? == "package").then_some(&parts[1..])?
    } else {
        parts
    };
    let decoded: Vec<String> = package_parts.iter().map(|part| decode(part)).collect();
    let name_parts: Vec<String> = if decoded.len() == 1 {
        decoded[0].split('/').map(str::to_owned).collect()
    } else {
        decoded
    };
    if name_parts.len() == 1 && is_segment(&name_parts[0]) {
        return Some(name_parts[0].clone());
    }
    if name_parts.len() == 2
        && name_parts[0].starts_with('@')
        && is_segment(&name_parts[0][1..])
        && is_segment(&name_parts[1])
    {
        return Some(name_parts.join("/"));
    }
    None
}
fn crates_name(parts: &[String]) -> Option<String> {
    if parts.len() >= 2 && parts[0] == "crates" {
        return unquoted_segment(&parts[1]);
    }
    if parts.len() == 4 && parts[..3] == ["api", "v1", "crates"] {
        return unquoted_segment(&parts[3]);
    }
    None
}
fn unquoted_segment(value: &str) -> Option<String> {
    let decoded = decode(value);
    is_segment(&decoded).then_some(decoded)
}
fn is_segment(value: &str) -> bool {
    !value.is_empty() && !value.contains('/')
}
fn decode(value: &str) -> String {
    percent_decode_str(value).decode_utf8_lossy().into_owned()
}
fn path_parts(parsed: &Url) -> Vec<String> {
    parsed
        .path_segments()
        .map(|segments| {
            segments
                .filter(|part| !part.is_empty())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}
fn npm_encode(value: &str) -> String {
    urlencoding::encode(value).replace("%40", "@")
}
