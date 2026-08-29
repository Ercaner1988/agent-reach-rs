//! Agent Reach Core — Platform-agnostic web/API reading capability layer
//!
//! This crate provides the foundational types and traits for the Agent Reach ecosystem:
//! - `Backend`: execution strategy trait (CLI subprocess, HTTP API, browser automation)
//! - `Channel`: platform reader trait (Twitter, Reddit, YouTube, RSS, etc.)
//! - `Config`: YAML-backed configuration with env var fallback
//! - `Doctor`: health check and availability probe system

pub mod backend;
pub mod cassette;
pub mod channel;
pub mod config;
pub mod doctor;
pub mod error;
pub mod media;

pub use backend::{Backend, BackendStatus};
pub use channel::{Channel, ChannelOutput};
pub use config::Config;
pub use doctor::{HealthCheck, HealthStatus};
pub use error::{Error, Result};
pub use media::{MediaInspector, MediaMetadata};
