use serde::{Deserialize, Serialize};

/// SearXNG response wrapper — the outermost JSON object returned by /search.
#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearxngResult>,
}

/// A single search result from SearXNG's API.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct SearxngResult {
    #[serde(default = "default_empty_string")]
    pub url: String,
    #[serde(default = "default_empty_string")]
    pub title: String,
    #[serde(default = "default_empty_string")]
    pub content: String,
    #[serde(default = "default_empty_string")]
    pub thumbnail: String,
    #[serde(default = "default_engine")]
    pub engine: String,
    #[serde(default)]
    pub parsed_url: Vec<String>,
    #[serde(default = "default_empty_string")]
    pub img_src: String,
    #[serde(default)]
    pub engines: Vec<String>,
    #[serde(default)]
    pub positions: Vec<u32>,
    #[serde(default = "default_zero_score")]
    pub score: f32,
    #[serde(default = "default_empty_string")]
    pub category: String,
}

fn default_empty_string() -> String {
    String::new()
}

fn default_engine() -> String {
    "unknown".to_string()
}

fn default_zero_score() -> f32 {
    0.0
}

/// The MCP-facing result schema — what gets serialized back through the tool response.
#[derive(Debug, Serialize)]
pub struct SearchResult {
    /// The title of the page
    pub title: String,
    /// The content of the page
    pub content: String,
    /// The URL of the page
    pub url: String,
    /// The engine from which the page was found
    pub engine: String,
    /// The score of the page. Higher score means more relevant to the query.
    pub score: f32,
}

/// Map a SearXNG result to our MCP-facing schema by field name.
/// Missing fields fall back to defaults: empty string for strings, 0 for numbers/floats.
impl From<SearxngResult> for SearchResult {
    fn from(r: SearxngResult) -> Self {
        Self {
            title: r.title,
            content: r.content,
            url: r.url,
            engine: r.engine,
            score: r.score,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapping_preserves_all_fields() {
        let raw = SearxngResult {
            url: "http://example.com".to_string(),
            title: "Example Title".to_string(),
            content: "Some content here".to_string(),
            thumbnail: "".to_string(),
            engine: "google".to_string(),
            parsed_url: vec![],
            img_src: "".to_string(),
            engines: vec![],
            positions: vec![],
            score: 42.5,
            category: "general".to_string(),
        };

        let result: SearchResult = raw.into();
        assert_eq!(result.title, "Example Title");
        assert_eq!(result.content, "Some content here");
        assert_eq!(result.url, "http://example.com");
        assert_eq!(result.engine, "google");
        assert!((result.score - 42.5).abs() < f32::EPSILON);
    }

    #[test]
    fn mapping_uses_defaults_for_missing_fields() {
        // All fields at their serde defaults (empty strings, zero score, "unknown" engine).
        let raw = SearxngResult {
            url: String::new(),
            title: String::new(),
            content: String::new(),
            thumbnail: String::new(),
            engine: "unknown".to_string(), // default_engine()
            parsed_url: vec![],
            img_src: String::new(),
            engines: vec![],
            positions: vec![],
            score: 0.0,
            category: String::new(),
        };

        let result: SearchResult = raw.into();
        assert_eq!(result.title, "");
        assert_eq!(result.content, "");
        assert_eq!(result.url, "");
        assert_eq!(result.engine, "unknown");
        assert!((result.score - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn search_response_deserializes_from_json() {
        let json = r#"{
            "results": [
                {"url": "http://a.com", "title": "A", "content": "C", "engine": "bing", "score": 1.0},
                {"url": "http://b.com", "title": "B"}
            ]
        }"#;

        let resp: SearchResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.results.len(), 2);
        assert_eq!(resp.results[0].engine, "bing");
        // Second result has no engine field — should default to "unknown"
        assert_eq!(resp.results[1].engine, "unknown");
    }

    #[test]
    fn search_result_serializes_to_json() {
        let results = vec![SearchResult {
            title: "T".to_string(),
            content: "C".to_string(),
            url: "http://x.com".to_string(),
            engine: "e".to_string(),
            score: 1.0,
        }];

        let json = serde_json::to_string(&results).unwrap();
        assert!(json.contains("T"));
        assert!(json.contains("C"));
        assert!(json.contains("http://x.com"));
    }
}
