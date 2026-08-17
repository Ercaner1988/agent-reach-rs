//! Xiaohongshu (RED) channel — notes, search, user feed
//!
//! Backends:
//! 1. xhs-web-scraper (HTTP) — public note HTML parsing / web API

use agent_reach_core::{
    backend::{Backend, BackendStatus},
    channel::{Channel, ChannelOutput, ChannelResult},
    doctor::HealthStatus,
    Config, Error,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Instant;

/// Xiaohongshu Web Scraper backend
pub struct XhsWebBackend;

#[async_trait]
impl Backend for XhsWebBackend {
    fn name(&self) -> &str {
        "xhs-web"
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
        let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

        let url = match action {
            "note" => {
                let note_id = args.first().ok_or_else(|| {
                    Error::BackendExecution(self.name().into(), "Missing note_id argument".into())
                })?;
                format!("https://www.xiaohongshu.com/explore/{}", note_id)
            }
            "search" => {
                let query = args.first().ok_or_else(|| {
                    Error::BackendExecution(self.name().into(), "Missing query argument".into())
                })?;
                format!(
                    "https://www.xiaohongshu.com/search_result?keyword={}",
                    urlencoding::encode(query)
                )
            }
            other => return Err(Error::UnsupportedAction("xiaohongshu".into(), other.into())),
        };

        let response = client
            .get(&url)
            .header("User-Agent", user_agent)
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

/// Xiaohongshu Channel orchestrator
pub struct XiaohongshuChannel {
    backends: Vec<Box<dyn Backend>>,
}

impl XiaohongshuChannel {
    pub fn new() -> Self {
        Self {
            backends: vec![Box::new(XhsWebBackend)],
        }
    }
}

impl Default for XiaohongshuChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Channel for XiaohongshuChannel {
    fn platform(&self) -> &str {
        "xiaohongshu"
    }

    fn actions(&self) -> Vec<String> {
        vec!["note".into(), "search".into()]
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

    #[tokio::test]
    async fn test_xhs_web_availability() {
        let backend = XhsWebBackend;
        let config = Config::default();
        let status = backend.is_available(&config).await;
        assert!(matches!(status, BackendStatus::Available));
    }
}
