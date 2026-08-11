//! Xueqiu (Snowball financial) channel — stock quotes, user posts, timelines
//!
//! Backends:
//! 1. xueqiu-web (HTTP) — Web API scraper for stock quotes and timeline

use agent_reach_core::{
    backend::{Backend, BackendStatus},
    channel::{Channel, ChannelOutput, ChannelResult},
    doctor::HealthStatus,
    Config, Error,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Instant;

/// Xueqiu Web API backend
pub struct XueqiuWebBackend;

#[async_trait]
impl Backend for XueqiuWebBackend {
    fn name(&self) -> &str {
        "xueqiu-web"
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
            "quote" | "stock" => {
                let symbol = args.first().ok_or_else(|| {
                    Error::BackendExecution(
                        self.name().into(),
                        "Missing stock symbol argument".into(),
                    )
                })?;
                format!(
                    "https://stock.xueqiu.com/v5/stock/quote.json?symbol={}",
                    symbol.to_uppercase()
                )
            }
            "timeline" | "user" => {
                let user_id = args.first().ok_or_else(|| {
                    Error::BackendExecution(self.name().into(), "Missing user_id argument".into())
                })?;
                format!(
                    "https://xueqiu.com/v4/statuses/user_timeline.json?user_id={}",
                    user_id
                )
            }
            "search" => {
                let query = args.first().ok_or_else(|| {
                    Error::BackendExecution(self.name().into(), "Missing query argument".into())
                })?;
                format!(
                    "https://xueqiu.com/query/v1/search/status.json?q={}",
                    urlencoding::encode(query)
                )
            }
            other => return Err(Error::UnsupportedAction("xueqiu".into(), other.into())),
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

/// Xueqiu Channel orchestrator
pub struct XueqiuChannel {
    backends: Vec<Box<dyn Backend>>,
}

impl XueqiuChannel {
    pub fn new() -> Self {
        Self {
            backends: vec![Box::new(XueqiuWebBackend)],
        }
    }
}

impl Default for XueqiuChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Channel for XueqiuChannel {
    fn platform(&self) -> &str {
        "xueqiu"
    }

    fn actions(&self) -> Vec<String> {
        vec![
            "quote".into(),
            "stock".into(),
            "timeline".into(),
            "search".into(),
        ]
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
    async fn test_xueqiu_web_availability() {
        let backend = XueqiuWebBackend;
        let config = Config::default();
        let status = backend.is_available(&config).await;
        assert!(matches!(status, BackendStatus::Available));
    }
}
