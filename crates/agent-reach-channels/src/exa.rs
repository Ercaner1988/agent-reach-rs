//! Exa Search channel — neural/semantic web search
//!
//! Backends:
//! 1. exa-api (HTTP) — requires exa_api_key

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
                    "numResults": 10,
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

/// Exa Channel orchestrator
pub struct ExaChannel {
    backends: Vec<Box<dyn Backend>>,
}

impl ExaChannel {
    pub fn new() -> Self {
        Self {
            backends: vec![Box::new(ExaApiBackend)],
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
    async fn test_exa_api_requires_credentials() {
        let backend = ExaApiBackend;
        let config = Config::default();
        let status = backend.is_available(&config).await;
        assert!(matches!(status, BackendStatus::RequiresConfig { .. }));
    }
}
