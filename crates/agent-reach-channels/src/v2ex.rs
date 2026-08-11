//! V2EX channel — topics, nodes, latest/hot feeds
//!
//! Backends:
//! 1. v2ex-api (HTTP) — Public REST API (v2ex.com/api/v2)

use agent_reach_core::{
    backend::{Backend, BackendStatus},
    channel::{Channel, ChannelOutput, ChannelResult},
    doctor::HealthStatus,
    Config, Error,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Instant;

/// V2EX REST API backend
pub struct V2exApiBackend;

#[async_trait]
impl Backend for V2exApiBackend {
    fn name(&self) -> &str {
        "v2ex-api"
    }

    async fn is_available(&self, _config: &Config) -> BackendStatus {
        BackendStatus::Available
    }

    async fn execute(
        &self,
        action: &str,
        args: &[String],
        _config: &Config,
    ) -> agent_reach_core::backend::BackendResult<Vec<u8>> {
        let client = reqwest::Client::new();
        // V2EX API requires personal access token for most v2 endpoints,
        // but v1 APIs for public topics work without auth.
        // We use the older v1 endpoints here for simplicity and broader access.
        let base_url = "https://www.v2ex.com/api";

        let url = match action {
            "topic" => {
                let topic_id = args.first().ok_or_else(|| {
                    Error::BackendExecution(self.name().into(), "Missing topic_id argument".into())
                })?;
                format!("{}/topics/show.json?id={}", base_url, topic_id)
            }
            "node" => {
                let node_name = args.first().ok_or_else(|| {
                    Error::BackendExecution(self.name().into(), "Missing node_name argument".into())
                })?;
                format!("{}/topics/show.json?node_name={}", base_url, node_name)
            }
            "hot" => format!("{}/topics/hot.json", base_url),
            "latest" => format!("{}/topics/latest.json", base_url),
            other => return Err(Error::UnsupportedAction("v2ex".into(), other.into())),
        };

        let response = client
            .get(&url)
            .header("User-Agent", "agent-reach-rs/0.1")
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
}

/// V2EX Channel orchestrator
pub struct V2exChannel {
    backends: Vec<Box<dyn Backend>>,
}

impl V2exChannel {
    pub fn new() -> Self {
        Self {
            backends: vec![Box::new(V2exApiBackend)],
        }
    }
}

impl Default for V2exChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Channel for V2exChannel {
    fn platform(&self) -> &str {
        "v2ex"
    }

    fn actions(&self) -> Vec<String> {
        vec!["topic".into(), "node".into(), "hot".into(), "latest".into()]
    }

    async fn execute(
        &self,
        action: &str,
        args: &[String],
        config: &Config,
    ) -> ChannelResult<ChannelOutput> {
        let start = Instant::now();

        let mut last_error = None;
        for backend in &self.backends {
            let status = backend.is_available(config).await;
            if !matches!(status, BackendStatus::Available) {
                tracing::debug!("Backend {} not available: {}", backend.name(), status);
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

        Err(last_error.unwrap_or_else(|| {
            Error::BackendUnavailable(self.platform().into(), "No backends available".into())
        }))
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

    #[tokio::test]
    async fn test_v2ex_api_availability() {
        let backend = V2exApiBackend;
        let config = Config::default();
        let status = backend.is_available(&config).await;
        assert!(matches!(status, BackendStatus::Available));
    }
}
