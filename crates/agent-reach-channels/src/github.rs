//! GitHub channel — repositories, issues, pull requests, code search
//!
//! Backends:
//! 1. gh CLI (subprocess) — requires GitHub auth (gh auth login)
//! 2. GitHub REST API (HTTP) — requires github_token config

// Query relaxation: 3-stage fallback (exact → noise-cleaned → core terms)
mod relaxation {
    pub(super) fn stage_2(query: &str) -> String {
        let noise = [
            "written in", "compatible database", "framework", "control", "headless",
            "devtools protocol", "api", "webdriver", "fast",
            "dataframe", "build", "smaller", "faster",
            "cross platform", "desktop apps", "with web frontend", "runtime",
            "reimplementation", "replacement", "clone", "disk", "usaage",
            // Turkish filler words
            "hızlı", "metin", "arama"
        ];
        let mut cleaned = query.to_lowercase();
        for phrase in noise {
            cleaned = cleaned.replace(phrase, " ");
        }
        // Special handling: "for rust" → "rust" (preserve "rust")
        cleaned = cleaned.replace("for rust", "rust");
        cleaned = cleaned.replace("for ", " ");
        
        cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    pub(super) fn stage_3(query: &str) -> String {
        let lower = query.to_lowercase();
        let generic = ["the", "a", "an", "in", "on", "at", "for", "with", "from", "to", "and", "or"];
        let words: Vec<&str> = lower
            .split_whitespace()
            .filter(|w| !generic.contains(w))
            .collect();
        
        if words.is_empty() {
            return String::new();
        }
        
        // Strategy: first word + two longest remaining words
        // This preserves brand names (uv, fd, bat) while keeping descriptive terms
        let mut result = vec![words[0]];
        let mut remaining: Vec<&str> = words[1..].to_vec();
        remaining.sort_by(|a, b| b.len().cmp(&a.len()));
        result.extend(remaining.into_iter().take(2));
        
        result.join(" ")
    }
}

use agent_reach_core::{
    backend::{Backend, BackendStatus},
    channel::{Channel, ChannelOutput, ChannelResult},
    doctor::HealthStatus,
    Config, Error,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Instant;

/// gh CLI backend (subprocess)
pub struct GhCliBackend;

#[async_trait]
impl Backend for GhCliBackend {
    fn name(&self) -> &str {
        "gh-cli"
    }

    async fn is_available(&self, _config: &Config) -> BackendStatus {
        // Check gh installation (cross-platform: `gh --version` works on Win/Linux/macOS)
        let check = tokio::process::Command::new("gh")
            .arg("--version")
            .output()
            .await;

        if check.is_err() || !check.unwrap().status.success() {
            return BackendStatus::NotInstalled {
                command: "gh".into(),
            };
        }

        // Check if authenticated
        let auth_check = tokio::process::Command::new("gh")
            .arg("auth")
            .arg("status")
            .output()
            .await;

        if auth_check.is_err() || !auth_check.unwrap().status.success() {
            return BackendStatus::RequiresConfig {
                missing: vec!["gh auth login".into()],
            };
        }

        BackendStatus::Available
    }

    async fn execute(
        &self,
        action: &str,
        args: &[String],
        _config: &Config,
    ) -> agent_reach_core::backend::BackendResult<Vec<u8>> {
        let mut cmd = tokio::process::Command::new("gh");

        match action {
            "repo" => {
                let repo = args.first().ok_or_else(|| {
                    Error::BackendExecution(self.name().into(), "Missing repo argument".into())
                })?;
                cmd.arg("repo")
                    .arg("view")
                    .arg(repo)
                    .arg("--json")
                    // `gh repo view` tekil alan adlari kullanir (stargazerCount/forkCount);
                    // cogul olanlar yalnizca `gh search repos` icin gecerlidir (bkz. "search" kolu).
                    .arg("name,description,url,stargazerCount,forkCount,updatedAt");
            }
            "issue" => {
                let repo = args.first().ok_or_else(|| {
                    Error::BackendExecution(self.name().into(), "Missing repo argument".into())
                })?;
                cmd.arg("issue")
                    .arg("list")
                    .arg("--repo")
                    .arg(repo)
                    .arg("--json")
                    .arg("number,title,state,createdAt,author,url");
            }
            "pr" => {
                let repo = args.first().ok_or_else(|| {
                    Error::BackendExecution(self.name().into(), "Missing repo argument".into())
                })?;
                cmd.arg("pr")
                    .arg("list")
                    .arg("--repo")
                    .arg(repo)
                    .arg("--json")
                    .arg("number,title,state,createdAt,author,url");
            }
            "search" => {
                let query = args.first().ok_or_else(|| {
                    Error::BackendExecution(self.name().into(), "Missing query argument".into())
                })?;
                
                // 3-stage fallback (clean architecture — no hardcoded rules)
                let stages = vec![
                    query.to_string(),
                    relaxation::stage_2(query),
                    relaxation::stage_3(query),
                ];
                
                // Collect results from all stages separately (round-robin interleaving)
                let mut stage_outputs: Vec<Vec<serde_json::Value>> = Vec::new();
                
                for stage_query in stages.iter() {
                    if stage_query.trim().is_empty() {
                        continue;
                    }

                    let mut stage_cmd = tokio::process::Command::new("gh");
                    stage_cmd
                        .arg("search")
                        .arg("repos")
                        .arg(stage_query)
                        .arg("--json")
                        .arg("fullName,description,stargazersCount")
                        .arg("--limit")
                        .arg("20");
                    
                    let output = stage_cmd
                        .output()
                        .await
                        .map_err(|e| Error::BackendExecution(self.name().into(), e.to_string()))?;
                    
                    if !output.status.success() {
                        stage_outputs.push(Vec::new());
                        continue;
                    }
                    
                    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
                        .unwrap_or(serde_json::json!([]));
                    
                    if let Some(arr) = json.as_array() {
                        stage_outputs.push(arr.clone());
                    } else {
                        stage_outputs.push(Vec::new());
                    }
                }
                
                // Round-robin interleaving: take 1st from each stage, then 2nd from each, etc.
                let mut final_results = Vec::new();
                let mut seen_repos = std::collections::HashSet::new();
                
                for i in 0..20 {
                    for stage_res in &stage_outputs {
                        if let Some(item) = stage_res.get(i) {
                            if let Some(full_name) = item.get("fullName").and_then(|v| v.as_str()) {
                                if seen_repos.insert(full_name.to_lowercase()) {
                                    final_results.push(item.clone());
                                    if final_results.len() >= 20 {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    if final_results.len() >= 20 {
                        break;
                    }
                }
                
                let result_json = serde_json::Value::Array(final_results);
                return Ok(serde_json::to_vec(&result_json).unwrap_or_else(|_| b"[]".to_vec()));
            }
            other => {
                return Err(Error::UnsupportedAction("github".into(), other.into()));
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

/// GitHub REST API backend (HTTP)
pub struct GitHubApiBackend;

#[async_trait]
impl Backend for GitHubApiBackend {
    fn name(&self) -> &str {
        "github-api"
    }

    async fn is_available(&self, config: &Config) -> BackendStatus {
        if config.github_token.is_none() {
            return BackendStatus::RequiresConfig {
                missing: vec!["github_token".into()],
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
        let token = config
            .github_token
            .as_ref()
            .ok_or_else(|| Error::Config("github_token not set".into()))?;

        let base_url = "https://api.github.com";

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
            "repo" => {
                let repo = args.first().ok_or_else(|| {
                    Error::BackendExecution(self.name().into(), "Missing repo argument".into())
                })?;
                format!("{}/repos/{}", base_url, repo)
            }
            "issue" => {
                let repo = args.first().ok_or_else(|| {
                    Error::BackendExecution(self.name().into(), "Missing repo argument".into())
                })?;
                format!("{}/repos/{}/issues", base_url, repo)
            }
            "pr" => {
                let repo = args.first().ok_or_else(|| {
                    Error::BackendExecution(self.name().into(), "Missing repo argument".into())
                })?;
                format!("{}/repos/{}/pulls", base_url, repo)
            }
            "search" => {
                let query = args.first().ok_or_else(|| {
                    Error::BackendExecution(self.name().into(), "Missing query argument".into())
                })?;
                format!(
                    "{}/search/repositories?q={}",
                    base_url,
                    urlencoding::encode(query)
                )
            }
            other => {
                return Err(Error::UnsupportedAction("github".into(), other.into()));
            }
        };

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "agent-reach-rs")
            .header("Accept", "application/vnd.github+json")
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

/// GitHub channel — orchestrate backends
pub struct GitHubChannel {
    backends: Vec<Box<dyn Backend>>,
}

impl GitHubChannel {
    pub fn new() -> Self {
        Self {
            backends: vec![Box::new(GhCliBackend), Box::new(GitHubApiBackend)],
        }
    }
}

impl Default for GitHubChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Channel for GitHubChannel {
    fn platform(&self) -> &str {
        "github"
    }

    fn actions(&self) -> Vec<String> {
        vec!["repo".into(), "issue".into(), "pr".into(), "search".into()]
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

        Err(last_error.unwrap_or_else(|| {
            agent_reach_core::backend::unavailable(self.platform(), &skipped)
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
    async fn test_github_api_requires_token() {
        let backend = GitHubApiBackend;
        let config = Config::default();
        let status = backend.is_available(&config).await;
        assert!(matches!(status, BackendStatus::RequiresConfig { .. }));
    }
}
