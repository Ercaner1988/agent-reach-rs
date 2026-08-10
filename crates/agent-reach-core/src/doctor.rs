//! Doctor — health check and availability probe system

use crate::backend::BackendStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Health check result for a single channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Channel/platform name
    pub channel: String,
    /// Overall status
    pub status: Status,
    /// Backend availability map (backend_name → status)
    pub backends: HashMap<String, BackendStatus>,
    /// Human-readable message
    pub message: String,
    /// Probe duration in milliseconds
    pub duration_ms: u64,
}

/// Overall health status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    /// At least one backend is available
    Ok,
    /// All backends require configuration
    Warning,
    /// All backends are unavailable
    Unavailable,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ok => write!(f, "✓ OK"),
            Self::Warning => write!(f, "⚠ WARNING"),
            Self::Unavailable => write!(f, "✗ UNAVAILABLE"),
        }
    }
}

impl HealthStatus {
    /// Create a new health status report
    pub fn new(
        channel: String,
        backends: HashMap<String, BackendStatus>,
        duration_ms: u64,
    ) -> Self {
        let (status, message) = Self::derive_status(&backends);
        Self {
            channel,
            status,
            backends,
            message,
            duration_ms,
        }
    }

    /// Derive overall status from backend availability
    fn derive_status(backends: &HashMap<String, BackendStatus>) -> (Status, String) {
        let available = backends
            .values()
            .any(|s| matches!(s, BackendStatus::Available));

        let all_require_config = backends
            .values()
            .all(|s| matches!(s, BackendStatus::RequiresConfig { .. }));

        if available {
            (Status::Ok, "At least one backend is available".into())
        } else if all_require_config {
            (Status::Warning, "All backends require configuration".into())
        } else {
            (Status::Unavailable, "No backends available".into())
        }
    }
}

/// Health check trait for channels
pub trait HealthCheck {
    /// Run health check and return status
    fn check(
        &self,
        config: &crate::Config,
    ) -> impl std::future::Future<Output = HealthStatus> + Send;
}
