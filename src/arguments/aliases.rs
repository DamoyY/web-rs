#![expect(
    clippy::impl_trait_in_params,
    clippy::missing_inline_in_public_items,
    reason = "Alias helpers are tiny and keep generic iterators readable."
)]
pub const SEARCH_FIELDS: &[FieldSpec] = &[
    FieldSpec::new("q", &["query", "queries"]),
    FieldSpec::new("recency", &["recencies"]),
    FieldSpec::new("domains", &["domain"]),
    FieldSpec::new("category", &["categories"]),
];
pub const OPEN_FIELDS: &[FieldSpec] = &[
    FieldSpec::new("url", &["urls"]),
    FieldSpec::new("chunk", &["chunks"]),
];
pub const FIND_FIELDS: &[FieldSpec] = &[
    FieldSpec::new("url", &["urls"]),
    FieldSpec::new("pattern", &["patterns"]),
    FieldSpec::new("snippet_tokens", &["snippet_token"]),
];
#[derive(Clone, Copy, Debug)]
pub struct FieldSpec {
    canonical: &'static str,
    aliases: &'static [&'static str],
}
impl FieldSpec {
    #[must_use]
    pub const fn new(canonical: &'static str, aliases: &'static [&'static str]) -> Self {
        Self { canonical, aliases }
    }
    #[must_use]
    pub fn matches(self, raw: &str) -> bool {
        let lowered = raw.to_ascii_lowercase();
        lowered == self.canonical || self.aliases.iter().any(|alias| lowered == *alias)
    }
    #[must_use]
    pub const fn canonical(self) -> &'static str {
        self.canonical
    }
}
#[must_use]
pub fn canonical_field<'field>(fields: &'field [FieldSpec], raw: &str) -> Option<&'field str> {
    fields
        .iter()
        .find(|field| field.matches(raw))
        .map(|field| field.canonical())
}
#[must_use]
pub fn canonical_category(raw: &str) -> Option<&'static str> {
    let lowered = raw.trim().to_ascii_lowercase();
    let normalized = lowered.replace(['_', '-'], " ");
    match normalized.as_str() {
        "company" | "companies" => Some("company"),
        "research paper" | "research papers" => Some("research paper"),
        "news" => Some("news"),
        "pdf" | "pdfs" => Some("pdf"),
        "personal site" | "personal sites" => Some("personal site"),
        "financial report" | "financial reports" => Some("financial report"),
        "people" | "person" => Some("people"),
        _ => None,
    }
}
#[must_use]
pub fn looks_like_request(fields: &[FieldSpec], keys: impl Iterator<Item = String>) -> bool {
    keys.into_iter()
        .any(|key| canonical_field(fields, &key).is_some())
}
