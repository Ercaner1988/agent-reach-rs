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

/// Reject a 200 response whose body carries none of the markers the payload must have.
///
/// `status().is_success()` is the whole check every channel does, so an upstream
/// that answers 200 with an error page is recorded as `"success": true` and the
/// caller never learns the data is missing. Three of those round-tripped in a
/// single audit: nitter.net's "is offline" notice, Xiaohongshu's login wall, and
/// Xiaoyuzhou's 找不到了 page. The status code says the request arrived; only the
/// body says it carried anything.
///
/// Phrased as "must contain" rather than "must not look like an error page" on
/// purpose — a junk-marker blocklist has to guess every way an upstream can fail,
/// while a payload marker is fixed by the action we asked for. If a search
/// response has no note and no tweet in it, it is not a search response, and the
/// reason it lacks one is the upstream's business, not ours.
pub fn require_payload(backend: &str, body: &[u8], markers: &[&str]) -> crate::Result<()> {
    let text = String::from_utf8_lossy(body);
    if markers.iter().any(|m| text.contains(m)) {
        return Ok(());
    }
    Err(crate::Error::BackendExecution(
        backend.into(),
        format!(
            "HTTP 200 but body carries no payload (none of {} present in {} bytes) \
             — likely a login wall, an offline notice, or a not-found page",
            markers.join(", "),
            body.len()
        ),
    ))
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

#[cfg(test)]
mod tests {
    use super::*;

    // Bodies below are excerpts from the three real 200-responses that were logged
    // as `"success": true` before this check existed.

    #[test]
    fn nitter_offline_page_is_rejected() {
        let body = br#"<title>nitter.net</title>
  <meta name="description" content="nitter.net is offline." />"#;
        assert!(require_payload("nitter", body, &["timeline-item", "tweet-content"]).is_err());
    }

    #[test]
    fn nitter_page_with_tweets_passes() {
        let body = br#"<div class="timeline-item"><div class="tweet-content">hello</div></div>"#;
        assert!(require_payload("nitter", body, &["timeline-item", "tweet-content"]).is_ok());
    }

    #[test]
    fn xiaoyuzhou_not_found_page_is_rejected() {
        let body = "<title class=\"jsx-4d9c\">找不到了</title>".as_bytes();
        assert!(require_payload("xiaoyuzhou-web", body, &["og:title"]).is_err());
    }

    #[test]
    fn xiaoyuzhou_real_podcast_page_passes() {
        let body = "<meta property=\"og:title\" content=\"AI Podcast | 小宇宙\"/>".as_bytes();
        assert!(require_payload("xiaoyuzhou-web", body, &["og:title"]).is_ok());
    }

    #[test]
    fn xiaohongshu_login_wall_is_rejected() {
        // The login wall still ships window.__INITIAL_STATE__, so only the note
        // fields separate it from a real search response.
        // `unreadEndNoteId` is the real trap: a `noteId` marker matches it as a
        // substring, so the wall passed until that marker was dropped.
        let body = br#"<script>window.__INITIAL_STATE__={"login":{"showLogIn":true},
            "unreadEndNoteId":"","validIds":{"noteIds":[]}}</script>"#;
        assert!(require_payload("xhs-web", body, &["note_id", "noteCard"]).is_err());
        assert!(
            require_payload("xhs-web", body, &["noteId"]).is_ok(),
            "substring trap changed shape — revisit the xiaohongshu markers"
        );
    }

    #[test]
    fn rejection_message_names_the_missing_markers() {
        let err = require_payload("xhs-web", b"<html></html>", &["note_id"]).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("note_id"), "marker not named: {msg}");
        assert!(msg.contains("HTTP 200"), "status not explained: {msg}");
    }
}
