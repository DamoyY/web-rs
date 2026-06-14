#![expect(
    clippy::exhaustive_enums,
    clippy::impl_trait_in_params,
    clippy::missing_inline_in_public_items,
    clippy::module_name_repetitions,
    reason = "Target names mirror direct-fetch protocol concepts."
)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResponseFormat {
    Text,
    MediaWikiApi,
    PackageRegistryJson,
}
#[derive(Clone, Debug)]
pub struct DirectFetchTarget {
    pub original_url: String,
    pub request_url: String,
    pub accept_header: Option<String>,
    pub required_content_type: Option<String>,
    pub similarity_probe_url: Option<String>,
    pub response_format: ResponseFormat,
    pub json_fields_last: Vec<String>,
}
impl DirectFetchTarget {
    #[must_use]
    pub fn text(original_url: impl Into<String>, request_url: impl Into<String>) -> Self {
        Self {
            original_url: original_url.into(),
            request_url: request_url.into(),
            accept_header: None,
            required_content_type: None,
            similarity_probe_url: None,
            response_format: ResponseFormat::Text,
            json_fields_last: Vec::new(),
        }
    }
    #[must_use]
    pub fn markdown_accept(url: &str) -> Self {
        let mut target = Self::text(url, url);
        target.accept_header = Some("text/markdown".to_owned());
        target.required_content_type = Some("text/markdown".to_owned());
        target
    }
    #[must_use]
    pub fn package(original_url: &str, request_url: String, fields_last: Vec<String>) -> Self {
        let mut target = Self::text(original_url, request_url);
        target.response_format = ResponseFormat::PackageRegistryJson;
        target.json_fields_last = fields_last;
        target
    }
    #[must_use]
    pub fn mediawiki(original_url: &str, request_url: String) -> Self {
        let mut target = Self::text(original_url, request_url);
        target.response_format = ResponseFormat::MediaWikiApi;
        target
    }
}
