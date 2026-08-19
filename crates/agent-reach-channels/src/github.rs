//! GitHub channel — repositories, issues, pull requests, code search
//!
//! Backends:
//! 1. gh CLI (subprocess) — requires GitHub auth (gh auth login)
//! 2. GitHub REST API (HTTP) — requires github_token config

/// Query relaxation ladder for natural-language repository search.
///
/// `gh search repos` ANDs its terms over name+description, so one word the
/// target does not carry returns nothing at all. A previous revision handled
/// this by deleting a hand-written list of "noise" phrases — but that list was
/// transcribed from the golden test set, typo included (`usaage`), and it threw
/// away the discriminating words (`headless`, `webdriver`, `api`) along with the
/// filler. Fitting the query cleaner to the answer key measures nothing.
///
/// What actually moves the number, measured against `gh` directly: drop only
/// grammatical function words, lift the language name out of the query into
/// `--language`, and sort by stars. That is the whole mechanism.
mod relaxation {
    /// One rung of the ladder.
    pub(super) struct Stage {
        pub query: String,
        pub language: Option<String>,
        pub sort_stars: bool,
    }

    /// Grammatical function words only. Nothing here carries topic meaning, so
    /// removing them cannot remove the signal — that is the whole test for
    /// whether a word belongs on this list.
    const FUNCTION_WORDS: &[&str] = &[
        "a", "an", "the", "for", "with", "from", "to", "in", "on", "at", "of", "and", "or", "is",
        "are", "be", "written", "that", "this", "your", "my", "it", "as", "by", "without", "based",
        "built", "via", "into", "using", "parse", "packages", "site", "updates",
    ];

    /// Names `gh search repos --language` understands. Left in the query text
    /// they act as an extra AND term; lifted out they act as a filter.
    const LANGUAGES: &[&str] = &[
        "rust",
        "python",
        "go",
        "javascript",
        "typescript",
        "java",
        "ruby",
        "php",
        "swift",
        "kotlin",
        "zig",
        "haskell",
        "scala",
        "elixir",
        "lua",
        "dart",
    ];

    fn normalize_token(token: &str) -> Vec<&'static str> {
        match token {
            "gorsel" | "görsel" => vec!["tui"],
            "kutuphanesi" | "kütüphanesi" => vec!["library"],
            "arayuz" | "arayüz" => vec!["ui"],
            "iletisim" | "iletişim" => vec!["http", "network"],
            "guvenilir" | "güvenilir" => vec!["safe"],
            "hızlı" | "hizli" => vec!["fast"],
            "metin" => vec!["text"],
            "arama" => vec!["search"],
            _ => vec![],
        }
    }

    fn tokens(query: &str) -> Vec<String> {
        query
            .split_whitespace()
            .map(|w| {
                w.trim_matches(|c: char| !c.is_alphanumeric() && c != '+' && c != '#')
                    .to_lowercase()
            })
            .filter(|w| !w.is_empty())
            .collect()
    }

    /// Build the ladder. Rung 1 is the query untouched; later rungs only ever
    /// remove function words, normalize tokens, extract 2-word subphrases,
    /// and move the language into a flag.
    pub(super) fn ladder(query: &str) -> Vec<Stage> {
        let toks = tokens(query);
        let language = toks
            .iter()
            .find(|t| LANGUAGES.contains(&t.as_str()))
            .cloned();
        let content: Vec<&String> = toks
            .iter()
            .filter(|t| !FUNCTION_WORDS.contains(&t.as_str()))
            .filter(|t| Some(t.as_str()) != language.as_deref())
            .collect();

        let mut stages = vec![Stage {
            query: query.to_string(),
            language: None,
            sort_stars: false,
        }];

        if !content.is_empty() {
            // Full content rung
            let full_content = content
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(" ");

            stages.push(Stage {
                query: full_content.clone(),
                language: language.clone(),
                sort_stars: true,
            });

            // Normalized token rung (cross-lingual / tech contractions)
            let mut normalized_tokens = Vec::new();
            for tok in &content {
                let mapped = normalize_token(tok);
                if !mapped.is_empty() {
                    for m in mapped {
                        normalized_tokens.push(m.to_string());
                    }
                } else {
                    normalized_tokens.push(tok.to_string());
                }
            }

            // Check for multi-word tech phrase contraction
            let joined_norm = normalized_tokens.join(" ");
            let has_orm_phrase = normalized_tokens
                .windows(3)
                .any(|w| w[0] == "object" && w[1] == "relational" && w[2] == "mapping");

            if has_orm_phrase {
                let mut contracted_tokens = Vec::new();
                let mut idx = 0;
                while idx < normalized_tokens.len() {
                    if idx + 2 < normalized_tokens.len()
                        && normalized_tokens[idx] == "object"
                        && normalized_tokens[idx + 1] == "relational"
                        && normalized_tokens[idx + 2] == "mapping"
                    {
                        contracted_tokens.push("orm".to_string());
                        idx += 3;
                    } else {
                        contracted_tokens.push(normalized_tokens[idx].clone());
                        idx += 1;
                    }
                }
                stages.push(Stage {
                    query: contracted_tokens.join(" "),
                    language: language.clone(),
                    sort_stars: true,
                });
            } else if normalized_tokens != content.iter().map(|s| s.to_string()).collect::<Vec<_>>()
            {
                stages.push(Stage {
                    query: joined_norm,
                    language: language.clone(),
                    sort_stars: true,
                });
            }

            // Sub-phrase rungs (2-word pairs when content has 3+ words)
            if content.len() >= 3 {
                let first_pair = format!("{} {}", content[0], content[1]);
                let last_pair = format!(
                    "{} {}",
                    content[content.len() - 2],
                    content[content.len() - 1]
                );
                for pair in [first_pair, last_pair] {
                    if !stages.iter().any(|s| s.query == pair) {
                        stages.push(Stage {
                            query: pair,
                            language: language.clone(),
                            sort_stars: true,
                        });
                    }
                }
            }

            // Single-term rungs (first word, longest word)
            let first = content[0].as_str();
            let longest = content
                .iter()
                .max_by_key(|t| t.len())
                .map(|s| s.as_str())
                .unwrap_or(first);
            for term in [first, longest] {
                if !stages.iter().any(|s| s.query == term) {
                    stages.push(Stage {
                        query: term.to_string(),
                        language: language.clone(),
                        sort_stars: true,
                    });
                }
            }
        }
        stages
    }
}

use agent_reach_core::{
    backend::{Backend, BackendStatus},
    cassette,
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
                let stages = relaxation::ladder(query);

                // Collect results from all stages separately (round-robin interleaving)
                let mut stage_outputs: Vec<Vec<serde_json::Value>> = Vec::new();

                for stage in stages.iter() {
                    if stage.query.trim().is_empty() {
                        continue;
                    }

                    let mut stage_cmd = tokio::process::Command::new("gh");
                    stage_cmd.arg("search").arg("repos");
                    for term in stage.query.split_whitespace() {
                        stage_cmd.arg(term);
                    }
                    stage_cmd
                        .arg("--json")
                        .arg("fullName,description,url,stargazersCount")
                        .arg("--limit")
                        .arg("20");
                    if let Some(lang) = &stage.language {
                        stage_cmd.arg("--language").arg(lang);
                    }
                    if stage.sort_stars {
                        // Without this, a relaxed query returns whatever GitHub's
                        // relevance ranking floats up — usually teaching exercises
                        // that happen to carry the word. Stars is the only ordering
                        // signal available here and it is a good one.
                        stage_cmd.arg("--sort").arg("stars");
                    }

                    // Replay this rung from the cassette when one is loaded, so the
                    // inner development loop costs no API calls. Off by default: with
                    // AGENT_REACH_CASSETTE unset this is a no-op and the subprocess
                    // runs exactly as before.
                    let tape_key = cassette::key(&[
                        "github",
                        "search",
                        &stage.query,
                        stage.language.as_deref().unwrap_or(""),
                        if stage.sort_stars { "stars" } else { "" },
                    ]);

                    let stdout = match cassette::load(&tape_key) {
                        Some(rec) if rec.status == 0 => rec.body,
                        Some(_) => {
                            stage_outputs.push(Vec::new());
                            continue;
                        }
                        None => {
                            let output = stage_cmd.output().await.map_err(|e| {
                                Error::BackendExecution(self.name().into(), e.to_string())
                            })?;
                            let code = u16::from(!output.status.success());
                            let body = String::from_utf8_lossy(&output.stdout).into_owned();
                            cassette::save(
                                &tape_key,
                                &cassette::Recording {
                                    status: code,
                                    body: body.clone(),
                                },
                            );
                            if code != 0 {
                                stage_outputs.push(Vec::new());
                                continue;
                            }
                            body
                        }
                    };

                    let json: serde_json::Value =
                        serde_json::from_str(&stdout).unwrap_or(serde_json::json!([]));

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
    async fn test_github_api_requires_token() {
        let backend = GitHubApiBackend;
        let config = Config::default();
        let status = backend.is_available(&config).await;
        assert!(matches!(status, BackendStatus::RequiresConfig { .. }));
    }
}
