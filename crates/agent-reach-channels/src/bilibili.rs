//! Bilibili channel — videos, search, dynamic (t.bilibili.com)
//!
//! Backends:
//! 1. bilibili-api-rest (HTTP) — minimal public API fallback
//! 2. bbdown (subprocess) — for video details/download (CLI)

use agent_reach_core::{
    backend::{Backend, BackendStatus},
    channel::{Channel, ChannelOutput, ChannelResult},
    doctor::HealthStatus,
    Config, Error,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Instant;

/// BBDown backend (CLI subprocess)
pub struct BbdownBackend;

#[async_trait]
impl Backend for BbdownBackend {
    fn name(&self) -> &str {
        "bbdown"
    }

    async fn is_available(&self, _config: &Config) -> BackendStatus {
        let check = tokio::process::Command::new("BBDown")
            .arg("--version")
            .output()
            .await;

        if check.is_ok() && check.unwrap().status.success() {
            BackendStatus::Available
        } else {
            BackendStatus::NotInstalled {
                command: "scoop install bbdown # or dotnet tool install -g BBDown".into(),
            }
        }
    }

    async fn execute(
        &self,
        action: &str,
        args: &[String],
        _config: &Config,
    ) -> agent_reach_core::backend::BackendResult<Vec<u8>> {
        let url_or_bvid = args.first().ok_or_else(|| {
            Error::BackendExecution(self.name().into(), "Missing url/bvid argument".into())
        })?;

        match action {
            "video" | "info" => {
                let output = tokio::process::Command::new("BBDown")
                    .arg(url_or_bvid)
                    .arg("--info-only")
                    .output()
                    .await
                    .map_err(|e| Error::BackendExecution(self.name().into(), e.to_string()))?;

                if !output.status.success() {
                    return Err(Error::BackendExecution(
                        self.name().into(),
                        String::from_utf8_lossy(&output.stderr).to_string(),
                    ));
                }

                Ok(output.stdout)
            }
            other => Err(Error::UnsupportedAction("bilibili".into(), other.into())),
        }
    }
}

/// Bilibili Public REST API backend
pub struct BilibiliRestBackend;

#[async_trait]
impl Backend for BilibiliRestBackend {
    fn name(&self) -> &str {
        "bilibili-rest"
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
            "video" | "info" => {
                let bvid = args.first().ok_or_else(|| {
                    Error::BackendExecution(self.name().into(), "Missing bvid argument".into())
                })?;
                format!(
                    "https://api.bilibili.com/x/web-interface/view?bvid={}",
                    bvid
                )
            }
            "search" => {
                let query = args.first().ok_or_else(|| {
                    Error::BackendExecution(self.name().into(), "Missing query argument".into())
                })?;
                format!(
                    "https://api.bilibili.com/x/web-interface/search/all/v2?keyword={}",
                    urlencoding::encode(query)
                )
            }
            other => return Err(Error::UnsupportedAction("bilibili".into(), other.into())),
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

/// Bilibili Channel orchestrator
pub struct BilibiliChannel {
    backends: Vec<Box<dyn Backend>>,
}

impl BilibiliChannel {
    pub fn new() -> Self {
        Self {
            backends: vec![Box::new(BbdownBackend), Box::new(BilibiliRestBackend)],
        }
    }
}

impl Default for BilibiliChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Channel for BilibiliChannel {
    fn platform(&self) -> &str {
        "bilibili"
    }

    fn actions(&self) -> Vec<String> {
        vec!["video".into(), "info".into(), "search".into()]
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
    async fn test_bilibili_rest_availability() {
        let backend = BilibiliRestBackend;
        let config = Config::default();
        let status = backend.is_available(&config).await;
        assert!(matches!(status, BackendStatus::Available));
    }
}
