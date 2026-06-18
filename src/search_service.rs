use crate::config::AppConfig;
use crate::http_client::{HttpClient, HttpError};
use crate::model::{SearchResponse, SearchResult};
use crate::query_builder::{SearchParams, build_search_url};

/// Orchestrates a search: builds the URL, fetches results from SearXNG, and maps them to MCP-facing schema.
#[derive(Clone)]
pub struct SearchService {
    client: HttpClient,
    base_url: String,
}

impl SearchService {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            client: HttpClient::new(config.http_timeout_secs, config.retry_count),
            base_url: config.base_url.clone(),
        }
    }

    /// Execute a search and return results.
    pub async fn search(&self, params: SearchParams) -> Result<Vec<SearchResult>, HttpError> {
        let url = build_search_url(&self.base_url, &params).map_err(|e| {
            // URL parse errors shouldn't happen in practice since we control the input,
            // but surface them as a server error for completeness.
            HttpError::ServerError {
                status: 500,
                body: format!("invalid URL: {}", e),
            }
        })?;

        let response = self
            .client
            .get_with_retry::<SearchResponse>(url.as_str())
            .await?;

        Ok(response
            .results
            .into_iter()
            .map(SearchResult::from)
            .collect())
    }
}
