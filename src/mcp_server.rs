use rmcp::{
    ServerHandler, handler::server::wrapper::Parameters, schemars, tool, tool_handler, tool_router,
};
use serde::Deserialize;

use crate::config::AppConfig;
use crate::query_builder::SearchParams;
use crate::search_service::SearchService;

/// Input schema for the `web_search` tool.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WebSearchParams {
    /// The search query string (required)
    pub query: String,
    /// Page number for pagination (default: 1)
    #[serde(default = "default_page")]
    pub page: u32,
    /// Language filter (e.g., "en", "fr") — empty means no filter
    #[serde(default = "default_language")]
    pub language: String,
    /// Category filters (e.g., ["general", "news"]) — empty means all categories
    #[serde(default)]
    pub categories: Vec<String>,
    /// Time range filter (e.g., "day", "week", "month", "year") — empty means no filter
    #[serde(default)]
    pub time_range: String,
}

fn default_page() -> u32 {
    1
}

fn default_language() -> String {
    "all".into()
}

/// The MCP server implementation.
#[derive(Clone)]
pub struct McpServer {
    service: SearchService,
}

impl McpServer {
    pub fn new(config: AppConfig) -> Self {
        Self {
            service: SearchService::new(&config),
        }
    }
}

/// Register the `web_search` tool using the rmcp macro.
#[tool_router]
impl McpServer {
    #[tool(
        description = r#"Performs a web search using SearXNG, ideal for general queries, news, articles and online content.
Supports multiple search categories, languages, time ranges and safe search filtering. 
Returns relevant results from multiple search engines combined."#
    )]
    async fn web_search(
        &self,
        Parameters(WebSearchParams {
            query,
            page,
            language,
            categories,
            time_range,
        }): Parameters<WebSearchParams>,
    ) -> String {
        let params = SearchParams {
            query,
            page,
            language,
            categories,
            time_range,
        };

        match self.service.search(params).await {
            Ok(results) => serde_json::to_string(&results).unwrap_or_default(),
            Err(e) => format!("search failed: {}", e),
        }
    }
}

#[tool_handler(name = "mcp-searxng", version = "0.1.0")]
impl ServerHandler for McpServer {}
