//! Reddit channel — subreddits, posts, comments, search
//!
//! Backends:
//! 1. praw (Python subprocess) — requires reddit_client_id, reddit_client_secret
//! 2. reddit API (HTTP) — OAuth2, requires same credentials

use agent_reach_core::{
    backend::{Backend, BackendStatus},
    channel::{Channel, ChannelOutput, ChannelResult},
    doctor::HealthStatus,
    Config, Error,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Instant;

/// PRAW backend (Python subprocess)
pub struct PrawBackend;

#[async_trait]
impl Backend for PrawBackend {
    fn name(&self) -> &str {
        "praw"
    }

    async fn is_available(&self, config: &Config) -> BackendStatus {
        // Check if Python and praw are available
        let check = tokio::process::Command::new("python3")
            .arg("-c")
            .arg("import praw")
            .output()
            .await;

        if check.is_err() || !check.unwrap().status.success() {
            return BackendStatus::NotInstalled {
                command: "python3 -m pip install praw".into(),
            };
        }

        let mut missing = Vec::new();
        if config.reddit_client_id.is_none() {
            missing.push("reddit_client_id".into());
        }
        if config.reddit_client_secret.is_none() {
            missing.push("reddit_client_secret".into());
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
        let client_id = config
            .reddit_client_id
            .as_ref()
            .ok_or_else(|| Error::Config("reddit_client_id not set".into()))?;
        let client_secret = config
            .reddit_client_secret
            .as_ref()
            .ok_or_else(|| Error::Config("reddit_client_secret not set".into()))?;

        let user_agent = config
            .reddit_user_agent
            .as_deref()
            .unwrap_or("agent-reach-rs/0.1");

        let script = match action {
            "subreddit" => {
                let sub = args.first().ok_or_else(|| {
                    Error::BackendExecution(self.name().into(), "Missing subreddit argument".into())
                })?;
                format!(
                    r#"
import praw
import json
reddit = praw.Reddit(
    client_id='{}',
    client_secret='{}',
    user_agent='{}'
)
sub = reddit.subreddit('{}')
posts = [{{
    'title': p.title,
    'author': str(p.author),
    'score': p.score,
    'url': p.url,
    'created_utc': p.created_utc,
    'num_comments': p.num_comments
}} for p in sub.hot(limit=10)]
print(json.dumps(posts))
"#,
                    client_id, client_secret, user_agent, sub
                )
            }
            "search" => {
                let query = args.first().ok_or_else(|| {
                    Error::BackendExecution(self.name().into(), "Missing query argument".into())
                })?;
                format!(
                    r#"
import praw
import json
reddit = praw.Reddit(
    client_id='{}',
    client_secret='{}',
    user_agent='{}'
)
results = [{{
    'title': p.title,
    'subreddit': str(p.subreddit),
    'score': p.score,
    'url': p.url
}} for p in reddit.subreddit('all').search('{}', limit=10)]
print(json.dumps(results))
"#,
                    client_id, client_secret, user_agent, query
                )
            }
            "post" => {
                let post_id = args.first().ok_or_else(|| {
                    Error::BackendExecution(self.name().into(), "Missing post_id argument".into())
                })?;
                format!(
                    r#"
import praw
import json
reddit = praw.Reddit(
    client_id='{}',
    client_secret='{}',
    user_agent='{}'
)
post = reddit.submission(id='{}')
post.comments.replace_more(limit=0)
data = {{
    'title': post.title,
    'author': str(post.author),
    'selftext': post.selftext,
    'score': post.score,
    'num_comments': post.num_comments,
    'comments': [{{
        'author': str(c.author),
        'body': c.body[:500],
        'score': c.score
    }} for c in post.comments.list()[:20]]
}}
print(json.dumps(data))
"#,
                    client_id, client_secret, user_agent, post_id
                )
            }
            other => {
                return Err(Error::UnsupportedAction("reddit".into(), other.into()));
            }
        };

        let output = tokio::process::Command::new("python3")
            .arg("-c")
            .arg(&script)
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

/// Reddit API backend (HTTP, OAuth2)
pub struct RedditApiBackend;

#[async_trait]
impl Backend for RedditApiBackend {
    fn name(&self) -> &str {
        "reddit-api"
    }

    async fn is_available(&self, config: &Config) -> BackendStatus {
        let mut missing = Vec::new();
        if config.reddit_client_id.is_none() {
            missing.push("reddit_client_id".into());
        }
        if config.reddit_client_secret.is_none() {
            missing.push("reddit_client_secret".into());
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
        let client_id = config
            .reddit_client_id
            .as_ref()
            .ok_or_else(|| Error::Config("reddit_client_id not set".into()))?;
        let client_secret = config
            .reddit_client_secret
            .as_ref()
            .ok_or_else(|| Error::Config("reddit_client_secret not set".into()))?;

        // Get OAuth2 token
        let token = self.get_oauth_token(client_id, client_secret).await?;

        let base_url = "https://oauth.reddit.com";

        let client = reqwest::Client::new();

        let url = match action {
            "subreddit" => {
                let sub = args.first().ok_or_else(|| {
                    Error::BackendExecution(self.name().into(), "Missing subreddit argument".into())
                })?;
                format!("{}/r/{}/hot.json?limit=10", base_url, sub)
            }
            "search" => {
                let query = args.first().ok_or_else(|| {
                    Error::BackendExecution(self.name().into(), "Missing query argument".into())
                })?;
                format!(
                    "{}/search.json?q={}&limit=10",
                    base_url,
                    urlencoding::encode(query)
                )
            }
            "post" => {
                let post_id = args.first().ok_or_else(|| {
                    Error::BackendExecution(self.name().into(), "Missing post_id argument".into())
                })?;
                format!("{}/comments/{}.json", base_url, post_id)
            }
            other => {
                return Err(Error::UnsupportedAction("reddit".into(), other.into()));
            }
        };

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "agent-reach-rs/0.1")
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

impl RedditApiBackend {
    async fn get_oauth_token(
        &self,
        client_id: &str,
        client_secret: &str,
    ) -> agent_reach_core::backend::BackendResult<String> {
        let client = reqwest::Client::new();

        let response = client
            .post("https://www.reddit.com/api/v1/access_token")
            .basic_auth(client_id, Some(client_secret))
            .form(&[("grant_type", "client_credentials")])
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Error::BackendExecution(
                self.name().into(),
                format!("OAuth failed: HTTP {}", response.status()),
            ));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        json.get("access_token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                Error::BackendExecution(
                    self.name().into(),
                    "No access_token in OAuth response".into(),
                )
            })
    }
}

/// Reddit channel — orchestrate backends
pub struct RedditChannel {
    backends: Vec<Box<dyn Backend>>,
}

impl RedditChannel {
    pub fn new() -> Self {
        Self {
            backends: vec![Box::new(PrawBackend), Box::new(RedditApiBackend)],
        }
    }
}

impl Default for RedditChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Channel for RedditChannel {
    fn platform(&self) -> &str {
        "reddit"
    }

    fn actions(&self) -> Vec<String> {
        vec!["subreddit".into(), "search".into(), "post".into()]
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
    async fn test_reddit_api_requires_credentials() {
        let backend = RedditApiBackend;
        let config = Config::default();
        let status = backend.is_available(&config).await;
        assert!(matches!(status, BackendStatus::RequiresConfig { .. }));
    }
}
