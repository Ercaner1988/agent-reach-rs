//! Backend execution strategy trait
//!
//! Backends represent different ways to access the same platform:
//! - CLI subprocess (twitter-cli, gh, yt-dlp)
//! - HTTP API (Jina Reader, Exa)
//! - Browser automation (OpenCLI via extension)
//!
//! Each channel can define multiple backends with priority order (first-choice → fallback).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Backend execution result
pub type BackendResult<T> = crate::Result<T>;

/// Backend availability status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackendStatus {
    /// Backend is available and configured
    Available,
    /// Backend exists but requires configuration (e.g., missing API key)
    RequiresConfig { missing: Vec<String> },
    /// Backend binary/dependency not installed
    NotInstalled { command: String },
    /// Backend check failed (timeout, error)
    Unavailable { reason: String },
}

impl fmt::Display for BackendStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Available => write!(f, "✓ available"),
            Self::RequiresConfig { missing } => {
                write!(f, "⚠ requires config: {}", missing.join(", "))
            }
            Self::NotInstalled { command } => write!(f, "✗ not installed: {}", command),
            Self::Unavailable { reason } => write!(f, "✗ unavailable: {}", reason),
        }
    }
}

/// Backend trait — execution strategy for a platform
#[async_trait]
pub trait Backend: Send + Sync {
    /// Backend identifier (e.g., "twitter-cli", "opencli", "jina-reader")
    fn name(&self) -> &str;

    /// Check if this backend is available
    async fn is_available(&self, config: &crate::Config) -> BackendStatus;

    /// Execute a query/command with this backend
    ///
    /// # Arguments
    /// - `action`: verb describing the operation (e.g., "search", "read", "fetch")
    /// - `args`: platform-specific arguments (e.g., query string, URL, user ID)
    /// - `config`: global config for API keys, cookies, proxy
    ///
    /// # Returns
    /// Raw output (JSON, text, or structured data depending on backend)
    async fn execute(
        &self,
        action: &str,
        args: &[String],
        config: &crate::Config,
    ) -> BackendResult<Vec<u8>>;
}

/// Convenience trait for backends that always return UTF-8 text
#[async_trait]
pub trait TextBackend: Backend {
    async fn execute_text(
        &self,
        action: &str,
        args: &[String],
        config: &crate::Config,
    ) -> BackendResult<String> {
        let bytes = self.execute(action, args, config).await?;
        String::from_utf8(bytes).map_err(|e| crate::Error::Decode(e.to_string()))
    }
}

// Blanket impl: any Backend can be used as TextBackend
impl<T: Backend> TextBackend for T {}
