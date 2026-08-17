//! DuckDuckGo search backend — quota-free GitHub repository search
//!
//! Uses `site:github.com <query>` to search GitHub via DuckDuckGo HTML scraping.

use agent_reach_core::{Backend, BackendStatus, Channel, Config, Error};
use async_trait::async_trait;
use regex::Regex;
use serde_json::Value;

pub struct DuckDuckGoBackend;

#[async_trait]
impl Backend for DuckDuckGoBackend {
    fn name(&self) -> &str {
        "duckduckgo"
    }

    async fn is_available(&self, _config: &Config) -> BackendStatus {
        // No API key required
        BackendStatus::Available
    }

    async fn execute(
        &self,
        action: &str,
        args: &[String],
        _config: &Config,
    ) -> Result<Vec<u8>, Error> {
        if action != "search" {
            return Err(Error::UnsupportedAction(self.name().into(), action.into()));
        }

        let query = args.first().ok_or_else(|| {
            Error::BackendExecution(self.name().into(), "Missing query argument".into())
        })?;

        // DuckDuckGo HTML search: site:github.com <query>
        let search_query = format!("site:github.com {}", query);
        let encoded_query = urlencoding::encode(&search_query);
        let url = format!("https://html.duckduckgo.com/html/?q={}", encoded_query);

        // HTTP GET request
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (compatible; AgentReach/0.1)")
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| Error::BackendExecution(self.name().into(), e.to_string()))?;

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::BackendExecution(self.name().into(), e.to_string()))?;

        let html = response
            .text()
            .await
            .map_err(|e| Error::BackendExecution(self.name().into(), e.to_string()))?;

        // Extract GitHub owner/repo slugs from HTML
        let repo_regex = Regex::new(r"github\.com/([a-zA-Z0-9_-]+/[a-zA-Z0-9_.-]+)")
            .map_err(|e| Error::BackendExecution(self.name().into(), e.to_string()))?;

        let mut repos = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for cap in repo_regex.captures_iter(&html) {
            if let Some(slug) = cap.get(1) {
                let slug_str = slug.as_str();
                
                // Filter out static GitHub pages (not repos)
                if slug_str.contains("/features")
                    || slug_str.contains("/pricing")
                    || slug_str.contains("/topics")
                    || slug_str.contains("/collections")
                    || slug_str.contains("/about")
                {
                    continue;
                }

                // Dedupe
                if seen.insert(slug_str.to_lowercase()) {
                    repos.push(Value::Object(
                        [
                            ("fullName".to_string(), Value::String(slug_str.to_string())),
                            ("source".to_string(), Value::String("duckduckgo".to_string())),
                        ]
                        .iter()
                        .cloned()
                        .collect(),
                    ));

                    if repos.len() >= 20 {
                        break;
                    }
                }
            }
        }

        let result_json = Value::Array(repos);
        Ok(serde_json::to_vec(&result_json).unwrap_or_else(|_| b"[]".to_vec()))
    }
}

/// DuckDuckGo Channel — orchestrates DuckDuckGo HTML scraping backend
pub struct DuckDuckGoChannel {
    backends: Vec<Box<dyn Backend>>,
}

impl DuckDuckGoChannel {
    pub fn new() -> Self {
        Self {
            backends: vec![Box::new(DuckDuckGoBackend)],
        }
    }
}

impl Default for DuckDuckGoChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Channel for DuckDuckGoChannel {
    fn name(&self) -> &str {
        "duckduckgo"
    }

    async fn execute(
        &self,
        action: &str,
        args: &[String],
        config: &Config,
    ) -> Result<agent_reach_core::ChannelOutput, Error> {
        // Try backend
        for backend in &self.backends {
            match backend.execute(action, args, config).await {
                Ok(data) => {
                    let json: Value = serde_json::from_slice(&data).unwrap_or(Value::Array(vec![]));
                    return Ok(agent_reach_core::ChannelOutput {
                        data: json,
                        metadata: serde_json::json!({
                            "backend": backend.name(),
                            "channel": self.name(),
                        }),
                    });
                }
                Err(e) => {
                    eprintln!("[DuckDuckGo] Backend '{}' failed: {}", backend.name(), e);
                    continue;
                }
            }
        }

        Err(Error::BackendExecution(
            self.name().into(),
            "All backends failed".into(),
        ))
    }

    async fn health_check(&self, config: &Config) -> agent_reach_core::HealthStatus {
        agent_reach_core::HealthStatus {
            healthy: true,
            backend_status: self
                .backends
                .iter()
                .map(|b| (b.name().to_string(), b.is_available(config)))
                .collect::<futures::stream::FuturesUnordered<_>>()
                .collect::<Vec<_>>()
                .await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_duckduckgo_search() {
        let channel = DuckDuckGoChannel::new();
        let config = Config::default();

        let result = channel
            .execute("search", &["rust http client".to_string()], &config)
            .await;

        assert!(result.is_ok(), "DuckDuckGo search should succeed");
        let output = result.unwrap();
        assert!(output.data.is_array(), "Output should be JSON array");
        let repos = output.data.as_array().unwrap();
        assert!(!repos.is_empty(), "Should find at least one repo");
    }
}
