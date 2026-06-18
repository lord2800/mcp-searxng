use clap::{Parser, ValueEnum};
use derive_more::Display;

#[derive(ValueEnum, Debug, Clone, Copy, Display)]
pub enum Transport {
    #[value(alias = "Stdio", alias = "stdio")]
    Stdio,
    #[value(alias = "HTTP", alias = "http")]
    HTTP,
}

#[derive(Debug, Clone, Parser)]
pub struct AppConfig {
    /// Base URL of the SearXNG instance (e.g., http://localhost:8080)
    #[arg(short, long, default_value = "http://localhost:8080")]
    pub base_url: String,

    /// Number of retry attempts for transient failures
    #[arg(short, long, default_value_t = 3)]
    pub retry_count: u32,

    /// Per-request HTTP timeout in seconds (default: 2)
    #[arg(short = 't', long, default_value_t = 2)]
    pub http_timeout_secs: u64,

    /// Transport mode: "stdio" or "http". Default is "stdio".
    #[arg(long, default_value_t = Transport::Stdio)]
    pub transport: Transport,

    // Bind address when using Streamable HTTP transport (default: 127.0.0.1)
    #[arg(short = 'a', long, default_value = "127.0.0.1")]
    pub bind_addr: String,

    /// Port to listen on when using Streamable HTTP transport (default: 8081)
    #[arg(short, long, default_value_t = 8081)]
    pub port: u16,
}
