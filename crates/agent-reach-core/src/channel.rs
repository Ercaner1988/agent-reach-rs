//! Channel trait — platform reader abstraction
//!
//! A channel represents a platform (Twitter, Reddit, YouTube) and orchestrates
//! multiple backends with fallback logic.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Channel operation result
pub type ChannelResult<T> = crate::Result<T>;

/// Channel output — structured result from a platform query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelOutput {
    /// Platform identifier (e.g., "twitter", "youtube", "rss")
    pub platform: String,
    /// Action performed (e.g., "search", "read", "fetch")
    pub action: String,
    /// Backend that successfully executed (e.g., "twitter-cli", "opencli")
    pub backend: String,
    /// Result data (JSON, text, or structured content)
    pub data: serde_json::Value,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
}

impl fmt::Display for ChannelOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} via {} ({}ms)",
            self.platform, self.action, self.backend, self.duration_ms
        )
    }
}

/// Channel trait — high-level platform reader
#[async_trait]
pub trait Channel: Send + Sync {
    /// Platform identifier (lowercase, e.g., "twitter", "youtube")
    fn platform(&self) -> &str;

    /// Available actions for this channel (e.g., ["search", "read", "user"])
    fn actions(&self) -> Vec<String>;

    /// Execute an action on this channel
    ///
    /// The channel tries backends in priority order (first-choice → fallback)
    /// until one succeeds or all fail.
    ///
    /// # Arguments
    /// - `action`: verb describing the operation
    /// - `args`: platform-specific arguments
    /// - `config`: global config
    ///
    /// # Returns
    /// Structured output with platform, backend, and result data
    async fn execute(
        &self,
        action: &str,
        args: &[String],
        config: &crate::Config,
    ) -> ChannelResult<ChannelOutput>;

    /// Health check: probe all backends and return availability report
    async fn health_check(&self, config: &crate::Config) -> crate::doctor::HealthStatus;
}
