//! Web channel — read arbitrary web pages via Jina Reader

use agent_reach_core::{
    backend::{Backend, BackendStatus},
    channel::{Channel, ChannelOutput, ChannelResult},
    doctor::HealthStatus,
    Config, Error,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Instant;

/// Jina Reader backend (r.jina.ai proxy)
pub struct JinaReaderBackend;

#[async_trait]
impl Backend for JinaReaderBackend {
    fn name(&self) -> &str {
        "jina-reader"
    }

    async fn is_available(&self, _config: &Config) -> BackendStatus {
        // Jina Reader is always available (no auth required)
        BackendStatus::Available
    }

    async fn execute(
        &self,
        action: &str,
        args: &[String],
        config: &Config,
    ) -> agent_reach_core::backend::BackendResult<Vec<u8>> {
        if action != "read" {
            return Err(Error::UnsupportedAction("web".into(), action.into()));
        }

        let url = args
            .first()
            .ok_or_else(|| Error::BackendExecution(self.name().into(), "Missing URL argument".into()))?;

        let jina_url = format!("https://r.jina.ai/{}", url);
        
        // Build client with optional proxy
        let mut client_builder = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30));

        if let Some(proxy_url) = &config.proxy {
            let proxy = reqwest::Proxy::all(proxy_url)
                .map_err(|e| Error::Config(format!("Invalid proxy: {}", e)))?;
            client_builder = client_builder.proxy(proxy);
        }

        let client = client_builder
            .build()
            .map_err(|e| Error::Network(e.to_string()))?;

        let response = client
            .get(&jina_url)
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Error::BackendExecution(
                self.name().into(),
                format!("HTTP {}: {}", response.status(), response.status().canonical_reason().unwrap_or("Unknown")),
            ));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        Ok(bytes.to_vec())
    }
}

/// Web channel — read web pages
pub struct WebChannel {
    backends: Vec<Box<dyn Backend>>,
}

impl WebChannel {
    pub fn new() -> Self {
        Self {
            backends: vec![Box::new(JinaReaderBackend)],
        }
    }
}

impl Default for WebChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Channel for WebChannel {
    fn platform(&self) -> &str {
        "web"
    }

    fn actions(&self) -> Vec<String> {
        vec!["read".into()]
    }

    async fn execute(
        &self,
        action: &str,
        args: &[String],
        config: &Config,
    ) -> ChannelResult<ChannelOutput> {
        let start = Instant::now();

        // Try backends in order until one succeeds
        let mut last_error = None;
        for backend in &self.backends {
            let status = backend.is_available(config).await;
            if !matches!(status, BackendStatus::Available) {
                tracing::debug!("Backend {} not available: {}", backend.name(), status);
                continue;
            }

            match backend.execute(action, args, config).await {
                Ok(data) => {
                    let text = String::from_utf8_lossy(&data).to_string();
                    return Ok(ChannelOutput {
                        platform: self.platform().into(),
                        action: action.into(),
                        backend: backend.name().into(),
                        data: serde_json::json!({ "text": text, "url": args.first() }),
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
            Error::BackendUnavailable(
                self.platform().into(),
                "No backends available".into(),
            )
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
    async fn test_jina_reader_availability() {
        let backend = JinaReaderBackend;
        let config = Config::default();
        let status = backend.is_available(&config).await;
        assert_eq!(status, BackendStatus::Available);
    }

    #[tokio::test]
    async fn test_web_channel_actions() {
        let channel = WebChannel::new();
        assert_eq!(channel.platform(), "web");
        assert_eq!(channel.actions(), vec!["read"]);
    }
}
