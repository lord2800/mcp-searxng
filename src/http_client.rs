use reqwest::Client;
use thiserror::Error;

/// Errors that can occur during HTTP operations.
#[derive(Debug, Error)]
pub enum HttpError {
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("server error (HTTP {status}): {body}")]
    ServerError { status: u16, body: String },

    #[error("JSON parse error: {0}")]
    Parse(#[from] serde_json::Error),
}

/// Classifies whether an HTTP response is retryable.
fn is_retryable(status: reqwest::StatusCode) -> bool {
    status.is_server_error()
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct HttpClient {
    client: Client,
    timeout_secs: u64,
    max_retries: u32,
}

impl HttpClient {
    pub fn new(timeout_secs: u64, max_retries: u32) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(timeout_secs))
                .build()
                .expect("should build reqwest client"),
            timeout_secs,
            max_retries,
        }
    }

    /// Execute a GET request with exponential backoff retry logic.
    /// Only network errors and HTTP 5xx are retryable; all other status codes
    /// (4xx, etc.) are treated as valid results and returned immediately.
    pub async fn get_with_retry<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
    ) -> Result<T, HttpError> {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            match self.client.get(url).send().await {
                Ok(response) => {
                    let status = response.status();

                    // Non-retryable: return the body as-is (even if it's an error page)
                    if !is_retryable(status) && !status.is_success() {
                        let body = response.text().await.unwrap_or_default();
                        return Err(HttpError::ServerError {
                            status: status.as_u16(),
                            body,
                        });
                    }

                    // Success — parse the JSON body
                    let text = response.text().await.map_err(|e| HttpError::Request(e))?;
                    match serde_json::from_str(&text) {
                        Ok(data) => return Ok(data),
                        Err(e) => last_error = Some(HttpError::Parse(e)),
                    }
                }
                Err(e) if e.is_timeout() || e.is_connect() => {
                    // Network error — retryable
                    last_error = Some(HttpError::Request(e));
                }
                Err(e) => {
                    // Other reqwest errors (e.g., invalid URL) are not retryable
                    return Err(HttpError::Request(e));
                }
            }

            // If we get here, the attempt failed and is retryable.
            if attempt < self.max_retries {
                let delay = Self::backoff_delay(attempt);
                tokio::time::sleep(delay).await;
            }
        }

        Err(last_error.unwrap_or_else(|| HttpError::ServerError {
            status: 500,
            body: "all retries exhausted".to_string(),
        }))
    }

    /// Exponential backoff: 1s, 2s, 4s, ... capped at 30s.
    fn backoff_delay(attempt: u32) -> std::time::Duration {
        let base = 1u64 << attempt; // 1, 2, 4, 8, 16, ...
        let capped = base.min(30);
        std::time::Duration::from_secs(capped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Backoff delay tests (synchronous, no network needed) ---

    #[test]
    fn backoff_delay_first_attempt() {
        assert_eq!(
            HttpClient::backoff_delay(0),
            std::time::Duration::from_secs(1)
        );
    }

    #[test]
    fn backoff_delay_exponential_growth() {
        assert_eq!(
            HttpClient::backoff_delay(1),
            std::time::Duration::from_secs(2)
        );
        assert_eq!(
            HttpClient::backoff_delay(2),
            std::time::Duration::from_secs(4)
        );
        assert_eq!(
            HttpClient::backoff_delay(3),
            std::time::Duration::from_secs(8)
        );
        assert_eq!(
            HttpClient::backoff_delay(4),
            std::time::Duration::from_secs(16)
        );
    }

    #[test]
    fn backoff_delay_capped_at_30() {
        // 2^5 = 32, capped to 30
        assert_eq!(
            HttpClient::backoff_delay(5),
            std::time::Duration::from_secs(30)
        );
        assert_eq!(
            HttpClient::backoff_delay(10),
            std::time::Duration::from_secs(30)
        );
    }

    // --- Network tests using a mock HTTP server ---

    #[tokio::test]
    async fn http_4xx_returns_immediately_without_retry() {
        use tokio::io::AsyncWriteExt;
        use tokio::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let url = format!("http://{}", addr);

        // Server that returns 404 on first request, then stays alive briefly.
        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                let response = b"HTTP/1.1 404 Not Found\r\nContent-Length: 5\r\nConnection: close\r\n\r\nerror";
                let _ = stream.write_all(response).await;
                // Keep the server alive briefly so reqwest can read.
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        });

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let client = HttpClient::new(2, 3); // 3 retries available
        let result: Result<serde_json::Value, HttpError> = client.get_with_retry(&url).await;

        match result {
            Err(HttpError::ServerError { status, body: _ }) => {
                assert_eq!(status, 404);
            }
            other => panic!("expected HttpError::ServerError(404), got {:?}", other),
        }
    }
}
