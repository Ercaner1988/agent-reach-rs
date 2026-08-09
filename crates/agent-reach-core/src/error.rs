//! Error types for Agent Reach

use thiserror::Error;

/// Agent Reach error type
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Backend not available
    #[error("Backend '{0}' is not available: {1}")]
    BackendUnavailable(String, String),

    /// Backend execution failed
    #[error("Backend '{0}' execution failed: {1}")]
    BackendExecution(String, String),

    /// Channel action not supported
    #[error("Channel '{0}' does not support action '{1}'")]
    UnsupportedAction(String, String),

    /// Decode error (invalid UTF-8, JSON parse failure, etc.)
    #[error("Decode error: {0}")]
    Decode(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic error
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;
