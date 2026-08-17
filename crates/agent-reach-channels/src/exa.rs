//! Exa Search channel — neural/semantic web search
//!
//! Backends, in priority order:
//! 1. exa-api (HTTP) — requires exa_api_key
//! 2. exa-mcp (HTTP) — Exa's public MCP endpoint, no key required
//!
//! Why two: the Python original reached Exa through `mcporter` pointed at
//! `https://mcp.exa.ai/mcp` and advertised it as "free, no API key needed". The
//! first Rust port replaced that with a direct `api.exa.ai` call, so a machine
//! without `EXA_API_KEY` — which was every machine here — lost search entirely.
//! The MCP endpoint is plain HTTP JSON-RPC, so talking to it directly restores
//! the keyless path without the npm dependency the original needed.

use agent_reach_core::{
    backend::{Backend, BackendStatus},
    channel::{Channel, ChannelOutput, ChannelResult},
    doctor::HealthStatus,
    Config, Error,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Instant;

/// Exa API backend
pub struct ExaApiBackend;

#[async_trait]
impl Backend for ExaApiBackend {
    fn name(&self) -> &str {
        "exa-api"
    }

    async fn is_available(&self, config: &Config) -> BackendStatus {
        if config.exa_api_key.is_some() {
            BackendStatus::Available
        } else {
            BackendStatus::RequiresConfig {
                missing: vec!["exa_api_key".into()],
            }
        }
    }

    async fn execute(
        &self,
        action: &str,
        args: &[String],
        config: &Config,
    ) -> agent_reach_core::backend::BackendResult<Vec<u8>> {
        let api_key = config
            .exa_api_key
            .as_ref()
            .ok_or_else(|| Error::Config("exa_api_key not set".into()))?;

        let query = args.first().ok_or_else(|| {
            Error::BackendExecution(self.name().into(), "Missing search query argument".into())
        })?;

        match action {
            "search" => {
                let client = reqwest::Client::new();
                let body = serde_json::json!({
                    "query": query,
                    "numResults": args.get(1).and_then(|s| s.parse::<u64>().ok()).unwrap_or(10),
                    "useAutoprompt": true
                });

                let response = client
                    .post("https://api.exa.ai/search")
                    .header("x-api-key", api_key)
                    .header("Content-Type", "application/json")
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| Error::Network(e.to_string()))?;

                if !response.status().is_success() {
                    return Err(Error::BackendExecution(
                        self.name().into(),
                        format!("HTTP {}", response.status()),
                    ));
                }

                let bytes = response
                    .bytes()
                    .await
                    .map_err(|e| Error::Network(e.to_string()))?;

                Ok(bytes.to_vec())
            }
            other => Err(Error::UnsupportedAction("exa".into(), other.into())),
        }
    }
}

/// Exa's public MCP endpoint — no API key.
///
/// Streamable-HTTP MCP: every call is a JSON-RPC POST to the same URL, and the
/// server replies with an SSE frame (`event: message` / `data: {...}`) even for
/// a single response, so the payload has to be unwrapped rather than parsed as
/// bare JSON. The session id comes back in the `mcp-session-id` header on
/// `initialize` and must be echoed on every later call.
pub struct ExaMcpBackend;

const EXA_MCP_URL: &str = "https://mcp.exa.ai/mcp";

/// Pull the JSON body out of an SSE frame, or parse it directly if the server
/// answered with plain JSON. Both shapes are legal for streamable HTTP.
fn parse_sse_json(body: &str) -> Option<serde_json::Value> {
    for line in body.lines() {
        if let Some(rest) = line.strip_prefix("data: ") {
            if let Ok(v) = serde_json::from_str(rest) {
                return Some(v);
            }
        }
    }
    serde_json::from_str(body).ok()
}

impl ExaMcpBackend {
    async fn rpc(
        client: &reqwest::Client,
        session: Option<&str>,
        body: serde_json::Value,
    ) -> agent_reach_core::backend::BackendResult<(Option<String>, Option<serde_json::Value>)> {
        let mut req = client
            .post(EXA_MCP_URL)
            .header("content-type", "application/json")
            .header("accept", "application/json, text/event-stream");
        if let Some(s) = session {
            req = req.header("mcp-session-id", s);
        }
        let resp = req
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(Error::BackendExecution(
                "exa-mcp".into(),
                format!("HTTP {}", resp.status()),
            ));
        }
        let sid = resp
            .headers()
            .get("mcp-session-id")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        let text = resp
            .text()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;
        Ok((sid, parse_sse_json(&text)))
    }
}

#[async_trait]
impl Backend for ExaMcpBackend {
    fn name(&self) -> &str {
        "exa-mcp"
    }

    /// No key, no local binary — nothing to check but the network, and probing
    /// that here would cost a round trip on every call. Report available and let
    /// `execute` surface a real failure if the endpoint is down.
    async fn is_available(&self, _config: &Config) -> BackendStatus {
        BackendStatus::Available
    }

    async fn execute(
        &self,
        action: &str,
        args: &[String],
        _config: &Config,
    ) -> agent_reach_core::backend::BackendResult<Vec<u8>> {
        if action != "search" {
            return Err(Error::UnsupportedAction("exa".into(), action.into()));
        }
        let query = args.first().ok_or_else(|| {
            Error::BackendExecution(self.name().into(), "Missing search query argument".into())
        })?;
        // The MCP tool passes num_results as args[1]; both backends used to ignore
        // it and always ask for 10, so the parameter was documented but inert.
        let num_results = args
            .get(1)
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(10);

        let client = reqwest::Client::new();
        let (session, _) = Self::rpc(
            &client,
            None,
            serde_json::json!({
                "jsonrpc": "2.0", "id": 1, "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": { "name": "agent-reach", "version": env!("CARGO_PKG_VERSION") }
                }
            }),
        )
        .await?;
        let session = session.ok_or_else(|| {
            Error::BackendExecution(self.name().into(), "no mcp-session-id returned".into())
        })?;

        Self::rpc(
            &client,
            Some(&session),
            serde_json::json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
        )
        .await?;

        // Tool name comes from the endpoint's own tools/list (measured, not guessed).
        let (_, reply) = Self::rpc(
            &client,
            Some(&session),
            serde_json::json!({
                "jsonrpc": "2.0", "id": 2, "method": "tools/call",
                "params": { "name": "web_search_exa", "arguments": { "query": query, "numResults": num_results } }
            }),
        )
        .await?;

        let reply = reply.ok_or_else(|| {
            Error::BackendExecution(self.name().into(), "unparseable MCP reply".into())
        })?;
        // A JSON-RPC error is a failure, not a result: returning it as content
        // would hand the caller an error message dressed up as search output.
        if let Some(err) = reply.get("error") {
            return Err(Error::BackendExecution(self.name().into(), err.to_string()));
        }
        let text = reply
            .pointer("/result/content")
            .and_then(|c| c.as_array())
            .map(|items| {
                // Return what the engine returned. An earlier revision reduced this
                // to a list of github.com slugs so a repo-finding benchmark would
                // score more easily; that turned a general web search tool into a
                // GitHub lookup and made every non-code query fail with
                // "empty result content". The benchmark is not the caller.
                items
                    .iter()
                    .filter_map(|i| i.get("text").and_then(|t| t.as_str()))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default();
        if text.is_empty() {
            return Err(Error::BackendExecution(
                self.name().into(),
                "empty result content".into(),
            ));
        }
        Ok(serde_json::to_vec(&serde_json::json!({ "text": text }))
            .map_err(|e| Error::Decode(e.to_string()))?)
    }
}

/// Exa Channel orchestrator
pub struct ExaChannel {
    backends: Vec<Box<dyn Backend>>,
}

impl ExaChannel {
    pub fn new() -> Self {
        // Key first when there is one — it is the account the user pays for and
        // carries their own limits; the public endpoint is the fallback.
        Self {
            backends: vec![Box::new(ExaApiBackend), Box::new(ExaMcpBackend)],
        }
    }
}

impl Default for ExaChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Channel for ExaChannel {
    fn platform(&self) -> &str {
        "exa"
    }

    fn actions(&self) -> Vec<String> {
        vec!["search".into()]
    }

    async fn execute(
        &self,
        action: &str,
        args: &[String],
        config: &Config,
    ) -> ChannelResult<ChannelOutput> {
        let start = Instant::now();

        let mut last_error = None;
        let mut skipped = Vec::new();
        for backend in &self.backends {
            let status = backend.is_available(config).await;
            if !matches!(status, BackendStatus::Available) {
                tracing::debug!("Backend {} not available: {}", backend.name(), status);
                skipped.push((backend.name().to_string(), status));
                continue;
            }

            match backend.execute(action, args, config).await {
                Ok(data) => {
                    let json_data: serde_json::Value = serde_json::from_slice(&data)
                        .unwrap_or_else(
                            |_| serde_json::json!({ "text": String::from_utf8_lossy(&data) }),
                        );
                    return Ok(ChannelOutput {
                        platform: self.platform().into(),
                        action: action.into(),
                        backend: backend.name().into(),
                        data: json_data,
                        duration_ms: start.elapsed().as_millis() as u64,
                    });
                }
                Err(e) => {
                    tracing::warn!("Backend {} failed: {}", backend.name(), e);
                    last_error = Some(e);
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| agent_reach_core::backend::unavailable(self.platform(), &skipped)))
    }

    async fn health_check(&self, config: &Config) -> HealthStatus {
        let start = Instant::now();
        let mut backends_status = HashMap::new();

        for backend in &self.backends {
            let status = backend.is_available(config).await;
            backends_status.insert(backend.name().into(), status);
        }

        HealthStatus::new(
            self.platform().into(),
            backends_status,
            start.elapsed().as_millis() as u64,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sse_json() {
        // What mcp.exa.ai actually sends: an event line, then the payload.
        let framed =
            "event: message\ndata: {\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{\"ok\":true}}\n\n";
        assert_eq!(parse_sse_json(framed).unwrap()["result"]["ok"], true);
        // Plain JSON is also legal for streamable HTTP.
        assert_eq!(parse_sse_json("{\"id\":1}").unwrap()["id"], 1);
        assert!(parse_sse_json("not json").is_none());
    }

    #[test]
    fn test_unavailable_names_the_missing_key() {
        // The whole point of C: the message has to say *why*.
        let err = agent_reach_core::backend::unavailable(
            "exa",
            &[(
                "exa-api".into(),
                BackendStatus::RequiresConfig {
                    missing: vec!["exa_api_key".into()],
                },
            )],
        );
        assert!(err.to_string().contains("exa_api_key"), "{err}");
    }

    #[tokio::test]
    async fn test_exa_api_requires_credentials() {
        let backend = ExaApiBackend;
        let config = Config::default();
        let status = backend.is_available(&config).await;
        assert!(matches!(status, BackendStatus::RequiresConfig { .. }));
    }
}
