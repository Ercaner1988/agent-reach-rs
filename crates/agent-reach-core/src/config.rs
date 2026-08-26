//! Configuration management
//!
//! Loads config from `~/.agent-reach/config.yaml` with environment variable fallback.

use anyhow::Context;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Agent Reach configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// Twitter auth token (cookie: auth_token)
    pub twitter_auth_token: Option<String>,
    /// Twitter ct0 token (cookie: ct0)
    pub twitter_ct0: Option<String>,
    /// Groq API key (for Whisper transcription)
    pub groq_api_key: Option<String>,
    /// OpenAI API key (for Whisper transcription fallback)
    pub openai_api_key: Option<String>,
    /// GitHub personal access token
    pub github_token: Option<String>,
    /// Reddit client ID (OAuth2)
    pub reddit_client_id: Option<String>,
    /// Reddit client secret (OAuth2)
    pub reddit_client_secret: Option<String>,
    /// Reddit user agent string
    pub reddit_user_agent: Option<String>,
    /// LinkedIn username
    pub linkedin_username: Option<String>,
    /// LinkedIn password
    pub linkedin_password: Option<String>,
    /// Exa API key (semantic search)
    pub exa_api_key: Option<String>,
    /// Network proxy (http://user:pass@ip:port)
    pub proxy: Option<String>,
    /// Channels to keep switched off, by name.
    ///
    /// A channel can be unwanted without being broken — a source whose terms the
    /// operator would rather not touch today, or one that is noisy for the work
    /// at hand. Turning it off here is one line of config instead of a rebuild,
    /// and the caller is told the channel is off rather than that it failed.
    #[serde(default)]
    pub disabled_channels: Vec<String>,
    /// Custom key-value store for platform-specific config
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Config {
    /// Load config from default location (~/.agent-reach/config.yaml)
    pub fn load() -> crate::Result<Self> {
        let path = Self::default_config_path()?;
        Self::load_from(&path)
    }

    /// Load config from a specific file
    pub fn load_from(path: &PathBuf) -> crate::Result<Self> {
        if !path.exists() {
            tracing::debug!(
                "Config file not found at {}, using defaults",
                path.display()
            );
            let mut config = Self::default();
            config.apply_env_overrides();
            return Ok(config);
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config from {}", path.display()))?;

        let mut config: Self = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse config YAML from {}", path.display()))?;

        // Override with environment variables (TWITTER_AUTH_TOKEN, GROQ_API_KEY, etc.)
        config.apply_env_overrides();

        Ok(config)
    }

    /// Save config to default location
    pub fn save(&self) -> crate::Result<()> {
        let path = Self::default_config_path()?;
        self.save_to(&path)
    }

    /// Save config to a specific file
    pub fn save_to(&self, path: &PathBuf) -> crate::Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create config directory {}", parent.display())
            })?;
        }

        let yaml = serde_yaml::to_string(self).context("Failed to serialize config to YAML")?;

        fs::write(path, yaml)
            .with_context(|| format!("Failed to write config to {}", path.display()))?;

        tracing::info!("Config saved to {}", path.display());
        Ok(())
    }

    /// Get config value by key (checks config file first, then env var)
    pub fn get(&self, key: &str) -> Option<String> {
        // Check struct fields first
        let from_struct = match key.to_lowercase().as_str() {
            "twitter_auth_token" => self.twitter_auth_token.clone(),
            "twitter_ct0" => self.twitter_ct0.clone(),
            "groq_api_key" => self.groq_api_key.clone(),
            "openai_api_key" => self.openai_api_key.clone(),
            "github_token" => self.github_token.clone(),
            "reddit_client_id" => self.reddit_client_id.clone(),
            "reddit_client_secret" => self.reddit_client_secret.clone(),
            "reddit_user_agent" => self.reddit_user_agent.clone(),
            "linkedin_username" => self.linkedin_username.clone(),
            "linkedin_password" => self.linkedin_password.clone(),
            "exa_api_key" => self.exa_api_key.clone(),
            "proxy" => self.proxy.clone(),
            _ => self
                .extra
                .get(key)
                .and_then(|v| v.as_str().map(|s| s.to_string())),
        };

        if from_struct.is_some() {
            return from_struct;
        }

        // Fallback to environment variable (uppercase)
        std::env::var(key.to_uppercase()).ok()
    }

    /// Set a config value by key
    pub fn set(&mut self, key: &str, value: String) {
        match key.to_lowercase().as_str() {
            "twitter_auth_token" => self.twitter_auth_token = Some(value),
            "twitter_ct0" => self.twitter_ct0 = Some(value),
            "groq_api_key" => self.groq_api_key = Some(value),
            "openai_api_key" => self.openai_api_key = Some(value),
            "github_token" => self.github_token = Some(value),
            "reddit_client_id" => self.reddit_client_id = Some(value),
            "reddit_client_secret" => self.reddit_client_secret = Some(value),
            "reddit_user_agent" => self.reddit_user_agent = Some(value),
            "linkedin_username" => self.linkedin_username = Some(value),
            "linkedin_password" => self.linkedin_password = Some(value),
            "exa_api_key" => self.exa_api_key = Some(value),
            "proxy" => self.proxy = Some(value),
            _ => {
                self.extra
                    .insert(key.to_string(), serde_json::Value::String(value));
            }
        }
    }

    /// Remove a config value by key. Returns whether a value was removed.
    pub fn unset(&mut self, key: &str) -> bool {
        match key.to_lowercase().as_str() {
            "twitter_auth_token" => self.twitter_auth_token.take().is_some(),
            "twitter_ct0" => self.twitter_ct0.take().is_some(),
            "groq_api_key" => self.groq_api_key.take().is_some(),
            "openai_api_key" => self.openai_api_key.take().is_some(),
            "github_token" => self.github_token.take().is_some(),
            "reddit_client_id" => self.reddit_client_id.take().is_some(),
            "reddit_client_secret" => self.reddit_client_secret.take().is_some(),
            "reddit_user_agent" => self.reddit_user_agent.take().is_some(),
            "linkedin_username" => self.linkedin_username.take().is_some(),
            "linkedin_password" => self.linkedin_password.take().is_some(),
            "exa_api_key" => self.exa_api_key.take().is_some(),
            "proxy" => self.proxy.take().is_some(),
            _ => self.extra.remove(key).is_some(),
        }
    }

    /// Apply environment variable overrides
    fn apply_env_overrides(&mut self) {
        // Comma separated, so a single shell variable can switch sources off for
        // one run without touching the file: AGENT_REACH_DISABLED_CHANNELS=reddit,quora
        if let Ok(val) = std::env::var("AGENT_REACH_DISABLED_CHANNELS") {
            self.disabled_channels = val
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect();
        }
        if let Ok(val) = std::env::var("TWITTER_AUTH_TOKEN") {
            self.twitter_auth_token = Some(val);
        }
        if let Ok(val) = std::env::var("TWITTER_CT0") {
            self.twitter_ct0 = Some(val);
        }
        if let Ok(val) = std::env::var("GROQ_API_KEY") {
            self.groq_api_key = Some(val);
        }
        if let Ok(val) = std::env::var("OPENAI_API_KEY") {
            self.openai_api_key = Some(val);
        }
        if let Ok(val) = std::env::var("GITHUB_TOKEN") {
            self.github_token = Some(val);
        }
        if let Ok(val) = std::env::var("REDDIT_CLIENT_ID") {
            self.reddit_client_id = Some(val);
        }
        if let Ok(val) = std::env::var("REDDIT_CLIENT_SECRET") {
            self.reddit_client_secret = Some(val);
        }
        if let Ok(val) = std::env::var("REDDIT_USER_AGENT") {
            self.reddit_user_agent = Some(val);
        }
        if let Ok(val) = std::env::var("LINKEDIN_USERNAME") {
            self.linkedin_username = Some(val);
        }
        if let Ok(val) = std::env::var("LINKEDIN_PASSWORD") {
            self.linkedin_password = Some(val);
        }
        if let Ok(val) = std::env::var("EXA_API_KEY") {
            self.exa_api_key = Some(val);
        }
        if let Ok(val) = std::env::var("HTTPS_PROXY").or_else(|_| std::env::var("HTTP_PROXY")) {
            self.proxy = Some(val);
        }
    }

    /// Default config file path: ~/.agent-reach/config.yaml
    /// Whether a channel may run. Compared case-insensitively and with
    /// surrounding whitespace ignored, because this list is typed by hand.
    pub fn channel_enabled(&self, channel: &str) -> bool {
        let wanted = channel.trim().to_lowercase();
        !self
            .disabled_channels
            .iter()
            .any(|c| c.trim().to_lowercase() == wanted)
    }

    pub fn default_config_path() -> crate::Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("", "", "agent-reach")
            .ok_or_else(|| crate::Error::Config("Could not determine home directory".into()))?;

        let config_dir = proj_dirs.config_dir();
        Ok(config_dir.join("config.yaml"))
    }

    /// Config directory path: ~/.agent-reach/
    pub fn config_dir() -> crate::Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("", "", "agent-reach")
            .ok_or_else(|| crate::Error::Config("Could not determine home directory".into()))?;

        Ok(proj_dirs.config_dir().to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_channel_is_on_unless_the_list_says_otherwise() {
        assert!(Config::default().channel_enabled("reddit"));

        let config = Config {
            disabled_channels: vec!["Reddit".into(), "  quora  ".into()],
            ..Default::default()
        };
        assert!(!config.channel_enabled("reddit"), "case must not matter");
        assert!(
            !config.channel_enabled("quora"),
            "stray spaces must not matter"
        );
        assert!(config.channel_enabled("github"), "others stay on");
    }

    #[test]
    fn a_channel_name_is_not_matched_by_a_prefix() {
        let config = Config {
            disabled_channels: vec!["red".into()],
            ..Default::default()
        };
        assert!(config.channel_enabled("reddit"));
    }

    #[test]
    fn test_config_get_set() {
        let mut config = Config::default();
        config.set("twitter_auth_token", "test_token".into());
        assert_eq!(config.get("twitter_auth_token"), Some("test_token".into()));
    }

    #[test]
    fn test_config_extra_fields() {
        let mut config = Config::default();
        config.set("custom_key", "custom_value".into());
        assert_eq!(config.get("custom_key"), Some("custom_value".into()));
    }

    #[test]
    fn test_config_unset() {
        let mut config = Config::default();
        config.set("custom_key", "custom_value".into());
        assert!(config.unset("custom_key"));
        assert!(!config.unset("custom_key"));
        assert_eq!(config.get("custom_key"), None);
    }
}
