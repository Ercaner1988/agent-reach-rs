//! Twitter channel — search tweets, read threads, fetch user timelines
//!
//! Backends:
//! 1. twitter-cli (subprocess) — requires auth_token + ct0 cookies
//! 2. nitter (HTTP scraper) — no auth, rate-limited

use agent_reach_core::{
    backend::{Backend, BackendStatus},
    channel::{Channel, ChannelOutput, ChannelResult},
    doctor::HealthStatus,
    Config, Error,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Instant;

/// twitter-cli backend (subprocess)
pub struct TwitterCliBackend;

#[async_trait]
impl Backend for TwitterCliBackend {
    fn name(&self) -> &str {
        "twitter-cli"
    }

    async fn is_available(&self, config: &Config) -> BackendStatus {
        // Check if twitter-cli is installed
        let check = tokio::process::Command::new("which")
            .arg("twitter")
            .output()
            .await;

        if check.is_err() || !check.unwrap().status.success() {
            return BackendStatus::NotInstalled {
                command: "twitter".into(),
            };
        }

        // Check config
        let mut missing = Vec::new();
        if config.twitter_auth_token.is_none() {
            missing.push("twitter_auth_token".into());
        }
        if config.twitter_ct0.is_none() {
            missing.push("twitter_ct0".into());
        }

        if missing.is_empty() {
            BackendStatus::Available
        } else {
            BackendStatus::RequiresConfig { missing }
        }
    }

    async fn execute(
        &self,
        action: &str,
        args: &[String],
        config: &Config,
    ) -> agent_reach_core::backend::BackendResult<Vec<u8>> {
        let auth_token = config
            .twitter_auth_token
            .as_ref()
            .ok_or_else(|| Error::Config("twitter_auth_token not set".into()))?;
        let ct0 = config
            .twitter_ct0
            .as_ref()
            .ok_or_else(|| Error::Config("twitter_ct0 not set".into()))?;

        let mut cmd = tokio::process::Command::new("twitter");
        cmd.env("TWITTER_AUTH_TOKEN", auth_token)
            .env("TWITTER_CT0", ct0);

        match action {
            "search" => {
                let query = args.first().ok_or_else(|| {
                    Error::BackendExecution(self.name().into(), "Missing query argument".into())
                })?;
                cmd.arg("search").arg(query);
            }
            "timeline" => {
                let user = args.first().ok_or_else(|| {
                    Error::BackendExecution(self.name().into(), "Missing user argument".into())
                })?;
                cmd.arg("timeline").arg(user);
            }
            "thread" => {
                let tweet_id = args.first().ok_or_else(|| {
                    Error::BackendExecution(self.name().into(), "Missing tweet_id argument".into())
                })?;
                cmd.arg("thread").arg(tweet_id);
            }
            other => {
                return Err(Error::UnsupportedAction("twitter".into(), other.into()));
            }
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

/// Nitter backend (HTTP scraper, no auth)
pub struct NitterBackend;

#[async_trait]
impl Backend for NitterBackend {
    fn name(&self) -> &str {
        "nitter"
    }

    async fn is_available(&self, _config: &Config) -> BackendStatus {
        // Nitter is always available (public instance)
        BackendStatus::Available
    }

    async fn execute(
        &self,
        action: &str,
        args: &[String],
        config: &Config,
    ) -> agent_reach_core::backend::BackendResult<Vec<u8>> {
        let base_url = "https://nitter.net";

        let client_builder = reqwest::Client::builder().timeout(std::time::Duration::from_secs(30));

        let client = if let Some(proxy_url) = &config.proxy {
            let proxy = reqwest::Proxy::all(proxy_url)
                .map_err(|e| Error::Config(format!("Invalid proxy: {}", e)))?;
            client_builder.proxy(proxy)
        } else {
            client_builder
        }
        .build()
        .map_err(|e| Error::Network(e.to_string()))?;

        let url = match action {
            "search" => {
                let query = args.first().ok_or_else(|| {
                    Error::BackendExecution(self.name().into(), "Missing query argument".into())
                })?;
                format!("{}/search?q={}", base_url, urlencoding::encode(query))
            }
            "timeline" => {
                let user = args.first().ok_or_else(|| {
                    Error::BackendExecution(self.name().into(), "Missing user argument".into())
                })?;
                format!("{}/{}", base_url, user.trim_start_matches('@'))
            }
            "thread" => {
                let tweet_id = args.first().ok_or_else(|| {
                    Error::BackendExecution(self.name().into(), "Missing tweet_id argument".into())
                })?;
                format!("{}/i/status/{}", base_url, tweet_id)
            }
            other => {
                return Err(Error::UnsupportedAction("twitter".into(), other.into()));
            }
        };

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Error::BackendExecution(
                self.name().into(),
                format!("HTTP {}", response.status()),
            ));
        }

        let html = response
            .text()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        // Simple HTML extraction (placeholder — real impl would use scraper crate)
        Ok(html.as_bytes().to_vec())
    }
}

/// Twitter channel — orchestrate backends
pub struct TwitterChannel {
    backends: Vec<Box<dyn Backend>>,
}

impl TwitterChannel {
    pub fn new() -> Self {
        Self {
            backends: vec![Box::new(TwitterCliBackend), Box::new(NitterBackend)],
        }
    }
}

impl Default for TwitterChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Channel for TwitterChannel {
    fn platform(&self) -> &str {
        "twitter"
    }

    fn actions(&self) -> Vec<String> {
        vec!["search".into(), "timeline".into(), "thread".into()]
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
                    let text = String::from_utf8_lossy(&data).to_string();
                    return Ok(ChannelOutput {
                        platform: self.platform().into(),
                        action: action.into(),
                        backend: backend.name().into(),
                        data: serde_json::json!({ "text": text }),
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
    async fn test_nitter_availability() {
        let backend = NitterBackend;
        let config = Config::default();
        let status = backend.is_available(&config).await;
        assert_eq!(status, BackendStatus::Available);
    }
}
