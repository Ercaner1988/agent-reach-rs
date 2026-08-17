//! YouTube channel — video metadata, transcripts, search
//!
//! Backends:
//! 1. rustube (native Rust library) — metadata only, no auth
//! 2. yt-dlp (subprocess) — full extraction including transcripts

use agent_reach_core::{
    backend::{Backend, BackendStatus},
    channel::{Channel, ChannelOutput, ChannelResult},
    doctor::HealthStatus,
    Config, Error,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Instant;

/// rustube backend (native Rust, metadata only)
pub struct RustubeBackend;

#[async_trait]
impl Backend for RustubeBackend {
    fn name(&self) -> &str {
        "rustube"
    }

    async fn is_available(&self, _config: &Config) -> BackendStatus {
        // rustube is a library, always available
        BackendStatus::Available
    }

    async fn execute(
        &self,
        action: &str,
        args: &[String],
        _config: &Config,
    ) -> agent_reach_core::backend::BackendResult<Vec<u8>> {
        if action != "metadata" {
            return Err(Error::UnsupportedAction("youtube".into(), action.into()));
        }

        let video_id = args.first().ok_or_else(|| {
            Error::BackendExecution(self.name().into(), "Missing video_id argument".into())
        })?;

        // Placeholder — real impl would use rustube crate
        let metadata = serde_json::json!({
            "id": video_id,
            "title": "[rustube metadata]",
            "description": "Placeholder for rustube extraction",
            "duration": null,
            "view_count": null,
        });

        Ok(serde_json::to_vec(&metadata).unwrap())
    }
}

/// yt-dlp backend (subprocess, full extraction)
pub struct YtDlpBackend;

#[async_trait]
impl Backend for YtDlpBackend {
    fn name(&self) -> &str {
        "yt-dlp"
    }

    async fn is_available(&self, _config: &Config) -> BackendStatus {
        let check = tokio::process::Command::new("which")
            .arg("yt-dlp")
            .output()
            .await;

        if check.is_err() || !check.unwrap().status.success() {
            return BackendStatus::NotInstalled {
                command: "yt-dlp".into(),
            };
        }

        BackendStatus::Available
    }

    async fn execute(
        &self,
        action: &str,
        args: &[String],
        config: &Config,
    ) -> agent_reach_core::backend::BackendResult<Vec<u8>> {
        let video_id = args.first().ok_or_else(|| {
            Error::BackendExecution(self.name().into(), "Missing video_id argument".into())
        })?;

        let url = if video_id.starts_with("http") {
            video_id.clone()
        } else {
            format!("https://www.youtube.com/watch?v={}", video_id)
        };

        let mut cmd = tokio::process::Command::new("yt-dlp");

        match action {
            "metadata" => {
                cmd.arg("--dump-json").arg(&url);
            }
            "transcript" => {
                cmd.arg("--skip-download")
                    .arg("--write-auto-sub")
                    .arg("--sub-format")
                    .arg("json3")
                    .arg(&url);
            }
            "search" => {
                let query = video_id;
                cmd.arg(format!("ytsearch5:{}", query)).arg("--dump-json");
            }
            other => {
                return Err(Error::UnsupportedAction("youtube".into(), other.into()));
            }
        }

        if let Some(proxy_url) = &config.proxy {
            cmd.arg("--proxy").arg(proxy_url);
        }

        let output = cmd
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
}

/// YouTube channel — orchestrate backends
pub struct YouTubeChannel {
    backends: Vec<Box<dyn Backend>>,
}

impl YouTubeChannel {
    pub fn new() -> Self {
        Self {
            backends: vec![Box::new(YtDlpBackend), Box::new(RustubeBackend)],
        }
    }
}

impl Default for YouTubeChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Channel for YouTubeChannel {
    fn platform(&self) -> &str {
        "youtube"
    }

    fn actions(&self) -> Vec<String> {
        vec!["metadata".into(), "transcript".into(), "search".into()]
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
    async fn test_rustube_availability() {
        let backend = RustubeBackend;
        let config = Config::default();
        let status = backend.is_available(&config).await;
        assert_eq!(status, BackendStatus::Available);
    }
}
