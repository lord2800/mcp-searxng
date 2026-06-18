use url::Url;

/// Search parameters matching the MCP tool input schema.
#[derive(Debug, Clone)]
pub struct SearchParams {
    pub query: String,
    pub page: u32,
    pub language: String,
    pub categories: Vec<String>,
    pub time_range: String,
}

/// Build a SearXNG search URL from the given parameters.
/// Parameter mapping:
///   query -> q
///   page -> pageno
///   language -> language
///   categories -> categories (comma-separated)
///   time_range -> time_range
pub fn build_search_url(base_url: &str, params: &SearchParams) -> Result<Url, url::ParseError> {
    let mut url = Url::parse(&format!("{}/search", base_url))?;

    // Pre-compute values that need to live long enough.
    let categories_str = if params.categories.is_empty() {
        None
    } else {
        Some(params.categories.join(","))
    };
    let pageno_str = params.page.to_string();

    {
        let mut query = url.query_pairs_mut();
        query.append_pair("q", &params.query);
        query.append_pair("pageno", &pageno_str);
        query.append_pair("language", &params.language);
        if let Some(ref cats) = categories_str {
            query.append_pair("categories", cats);
        }
        query.append_pair("time_range", &params.time_range);
        query.append_pair("format", "json");
    }

    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_url_maps_all_params() {
        let params = SearchParams {
            query: "rust mcp".to_string(),
            page: 3,
            language: "en".to_string(),
            categories: vec!["general".to_string(), "news".to_string()],
            time_range: "week".to_string(),
        };

        let url = build_search_url("http://localhost:8080", &params).unwrap();
        assert_eq!(url.host_str(), Some("localhost"));
        assert_eq!(url.port(), Some(8080));
        assert_eq!(url.path(), "/search");

        // Check query parameters.
        let pairs: std::collections::HashMap<_, _> = url.query_pairs().collect();
        assert_eq!(pairs["q"], "rust mcp");
        assert_eq!(pairs["pageno"], "3");
        assert_eq!(pairs["language"], "en");
        assert_eq!(pairs["categories"], "general,news");
        assert_eq!(pairs["time_range"], "week");
        assert_eq!(pairs["format"], "json");
    }

    #[test]
    fn build_url_uses_defaults_for_optional_fields() {
        let params = SearchParams {
            query: "hello".to_string(),
            page: 1,
            language: "all".to_string(),
            categories: vec![],
            time_range: "".to_string(),
        };

        let url = build_search_url("http://example.com", &params).unwrap();
        let pairs: std::collections::HashMap<_, _> = url.query_pairs().collect();

        assert_eq!(pairs["q"], "hello");
        assert!(!pairs.contains_key("categories")); // empty categories omitted
        assert_eq!(pairs["time_range"], "");
    }

    #[test]
    fn build_url_handles_special_chars_in_query() {
        let params = SearchParams {
            query: "a+b c&d".to_string(),
            page: 1,
            language: "all".to_string(),
            categories: vec![],
            time_range: "".to_string(),
        };

        let url = build_search_url("http://example.com", &params).unwrap();
        // The query string should be URL-encoded.
        assert!(url.query().unwrap().contains("%2B")); // + → %2B
    }

    #[test]
    fn build_url_multiple_categories_joined_with_comma() {
        let params = SearchParams {
            query: "test".to_string(),
            page: 1,
            language: "all".to_string(),
            categories: vec![
                "general".to_string(),
                "images".to_string(),
                "news".to_string(),
            ],
            time_range: "".to_string(),
        };

        let url = build_search_url("http://example.com", &params).unwrap();
        let pairs: std::collections::HashMap<_, _> = url.query_pairs().collect();
        assert_eq!(pairs["categories"], "general,images,news");
    }

    #[test]
    fn build_url_invalid_base_returns_error() {
        let params = SearchParams {
            query: "x".to_string(),
            page: 1,
            language: "all".to_string(),
            categories: vec![],
            time_range: "".to_string(),
        };

        assert!(build_search_url("not-a-valid-url", &params).is_err());
    }
}
