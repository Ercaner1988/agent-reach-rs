use agent_reach_channels::{RssChannel, WebChannel};
use agent_reach_core::{Channel, Config};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: Option<String>,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .init();

    run_stdio().await
}

async fn run_stdio() -> Result<()> {
    let stdin = BufReader::new(io::stdin());
    let mut lines = stdin.lines();
    let mut stdout = io::stdout();

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<JsonRpcRequest>(&line) {
            Ok(request) => handle_request(request).await,
            Err(error) => JsonRpcResponse {
                jsonrpc: "2.0",
                id: None,
                result: None,
                error: Some(JsonRpcError {
                    code: -32700,
                    message: format!("parse error: {error}"),
                }),
            },
        };

        let encoded = serde_json::to_string(&response)?;
        stdout.write_all(encoded.as_bytes()).await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;
    }

    Ok(())
}

async fn handle_request(request: JsonRpcRequest) -> JsonRpcResponse {
    let id = request.id.clone();
    if request.jsonrpc.as_deref() != Some("2.0") {
        return error(id, -32600, "invalid JSON-RPC version");
    }

    let result = match request.method.as_str() {
        "initialize" => Ok(initialize_result()),
        "tools/list" => Ok(tools_list_result()),
        "tools/call" => call_tool(request.params).await,
        other => Err(anyhow::anyhow!("method not found: {}", other)),
    };

    match result {
        Ok(value) => JsonRpcResponse {
            jsonrpc: "2.0",
            id,
            result: Some(value),
            error: None,
        },
        Err(err) => error(id, -32603, &err.to_string()),
    }
}

fn error(id: Option<Value>, code: i64, message: &str) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0",
        id,
        result: None,
        error: Some(JsonRpcError {
            code,
            message: message.to_string(),
        }),
    }
}

fn initialize_result() -> Value {
    json!({
        "protocolVersion": "2024-11-05",
        "serverInfo": {"name": "agent-reach-mcp", "version": env!("CARGO_PKG_VERSION")},
        "capabilities": {"tools": {}}
    })
}

fn tools_list_result() -> Value {
    json!({
        "tools": [
            {
                "name": "web_read",
                "description": "Read a web page through the Agent Reach web channel.",
                "inputSchema": {
                    "type": "object",
                    "properties": {"url": {"type": "string"}},
                    "required": ["url"]
                }
            },
            {
                "name": "rss_fetch",
                "description": "Fetch and parse an RSS 2.0 or Atom feed URL.",
                "inputSchema": {
                    "type": "object",
                    "properties": {"url": {"type": "string"}},
                    "required": ["url"]
                }
            },
            {
                "name": "rss_parse",
                "description": "Parse RSS 2.0 or Atom XML supplied as text.",
                "inputSchema": {
                    "type": "object",
                    "properties": {"xml": {"type": "string"}},
                    "required": ["xml"]
                }
            },
            {
                "name": "exa_search",
                "description": "Search the web through Exa API using exa_api_key from Agent Reach config.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": {"type": "string"},
                        "num_results": {"type": "integer", "minimum": 1, "maximum": 10}
                    },
                    "required": ["query"]
                }
            }
        ]
    })
}

async fn call_tool(params: Value) -> Result<Value> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .context("tools/call requires params.name")?;
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let config = Config::load().unwrap_or_default();

    let data = match name {
        "web_read" => {
            let url = required_string(&arguments, "url")?;
            WebChannel::new()
                .execute("read", &[url], &config)
                .await?
                .data
        }
        "rss_fetch" => {
            let url = required_string(&arguments, "url")?;
            RssChannel::new()
                .execute("fetch", &[url], &config)
                .await?
                .data
        }
        "rss_parse" => {
            let xml = required_string(&arguments, "xml")?;
            RssChannel::new()
                .execute("parse", &[xml], &config)
                .await?
                .data
        }
        "exa_search" => exa_search(&arguments, &config).await?,
        other => bail!("unknown tool: {other}"),
    };

    Ok(json!({
        "content": [{"type": "text", "text": serde_json::to_string_pretty(&data)?}],
        "isError": false
    }))
}

fn required_string(arguments: &Value, key: &str) -> Result<String> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .with_context(|| format!("missing required string argument: {key}"))
}

async fn exa_search(arguments: &Value, config: &Config) -> Result<Value> {
    let api_key = config
        .exa_api_key
        .clone()
        .context("exa_api_key is not configured")?;
    let query = required_string(arguments, "query")?;
    let num_results = arguments
        .get("num_results")
        .and_then(Value::as_u64)
        .unwrap_or(5)
        .clamp(1, 10);

    let response = reqwest::Client::new()
        .post("https://api.exa.ai/search")
        .header("x-api-key", api_key)
        .json(&json!({"query": query, "numResults": num_results}))
        .send()
        .await
        .context("Exa request failed")?;
    let status = response.status();
    let body = response
        .text()
        .await
        .context("failed to read Exa response")?;

    if !status.is_success() {
        bail!("Exa API returned HTTP {}: {}", status, body);
    }

    serde_json::from_str(&body).context("Exa API returned invalid JSON")
}
