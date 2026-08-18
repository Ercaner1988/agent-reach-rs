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

/// Build the error for "every backend was skipped".
///
/// `is_available` already computes exactly why a backend cannot run —
/// `RequiresConfig { missing }`, `NotInstalled { command }`, and so on. Channels
/// used to log that to `tracing::debug!` and then return a bare
/// `"No backends available"`, so the one fact the caller needed never reached
/// them: an unset `exa_api_key` looked identical to a network outage. Fold the
/// statuses into the message instead — they already know how to print themselves.
pub fn unavailable(platform: &str, skipped: &[(String, BackendStatus)]) -> crate::Error {
    let detail = if skipped.is_empty() {
        "no backends registered".to_string()
    } else {
        skipped
            .iter()
            .map(|(name, status)| format!("{name} {status}"))
            .collect::<Vec<_>>()
            .join("; ")
    };
    crate::Error::BackendUnavailable(platform.into(), detail)
}

/// Check whether a binary is on `PATH`, cross-platform (`where` on Windows, `which` elsewhere).
pub async fn binary_on_path(command: &str) -> bool {
    let finder = if cfg!(windows) { "where" } else { "which" };
    matches!(
        tokio::process::Command::new(finder)
            .arg(command)
            .output()
            .await,
        Ok(out) if out.status.success()
    )
}

/// First Python interpreter found on `PATH` (`python3`, then `python`).
pub async fn python_command() -> Option<&'static str> {
    for cmd in ["python3", "python"] {
        if let Ok(out) = tokio::process::Command::new(cmd)
            .arg("--version")
            .output()
            .await
        {
            if out.status.success() {
                return Some(cmd);
            }
        }
    }
    None
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
