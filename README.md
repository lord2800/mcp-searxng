# mcp-searxng

An MCP (Model Context Protocol) server that exposes a `web_search` tool backed by [SearXNG](https://github.com/searxng/searxng).

## Features

- **Web search via SearXNG** — queries any self-hosted or public SearXNG instance
- **Exponential backoff retry** — only network errors and HTTP 5xx are retried; 4xx responses are returned immediately
- **Configurable timeout & retries** — set per-request timeout and max retry count via CLI flags
- **Graceful shutdown** — Ctrl-C / SIGTERM signals trigger clean shutdown

## Building

```bash
cargo build --release
```

The binary will be at `target/release/mcp-searxng`.

## Usage

### CLI Options

| Flag | Description | Default |
|------|-------------|---------|
| `--transport` | Which transport to use (`http` or `stdio`) | `stdio` |
| `-b, --base-url` | Base URL of the SearXNG instance | `http://localhost:8080` |
| `-r, --retry-count` | Number of retry attempts for transient failures | `3` |
| `-t, --http-timeout-secs` | Per-request HTTP timeout in seconds | `2` |
| `-a, --bind-addr` | Bind address for HTTP transport | `127.0.0.1` |
| `-p, --port` | Port for HTTP transport | `8081` |

### Running

```bash
# With defaults (SearXNG at localhost:8080)
cargo run

# Custom SearXNG instance
cargo run -- -b http://my-server:8080 -r 5 -t 3
```

The server uses stdio transport — pipe it to an MCP client like Claude Desktop, Cursor, or VS Code.

### Configuring in Claude Desktop

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "web-search": {
      "command": "path/to/mcp-searxng",
      "args": ["-b", "http://localhost:8080"]
    }
  }
}
```

## Tool Schema

The `web_search` tool accepts the following parameters:

| Parameter | Type | Required | Description | Default |
|-----------|------|----------|-------------|---------|
| `query` | string | yes | The search query string | — |
| `page` | integer | no | Page number for pagination | 1 |
| `language` | string | no | Language filter (e.g., "en", "fr") | 'all' |
| `categories` | array\<string\> | no | Category filters (e.g., ["general", "news"]) | all categories |
| `time_range` | string | no | Time range ("day", "week", "month", "year") | empty |

## Retry Policy

- **Retryable**: network errors (timeout, connection refused) and HTTP 5xx responses
- **Non-retryable**: HTTP 4xx responses — returned immediately as valid results
- **Backoff**: exponential starting at 1s, doubling each attempt, capped at 30s

## License

AGPL-3.0-or-later
