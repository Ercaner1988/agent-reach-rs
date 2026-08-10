//! Doctor subcommand — check platform availability and health

use agent_reach_channels::{RssChannel, TwitterChannel, WebChannel};
use agent_reach_core::{BackendStatus, Channel, Config};
use anyhow::Result;
use serde_json::json;
use std::collections::HashMap;
use std::time::Instant;

pub async fn doctor(json_output: bool) -> Result<()> {
    let start = Instant::now();
    let config = Config::load().unwrap_or_default();

    // Run health checks for all available channels
    let web_status = WebChannel::new().health_check(&config).await;
    let rss_status = RssChannel::new().health_check(&config).await;
    let twitter_status = TwitterChannel::new().health_check(&config).await;

    let mut results = HashMap::new();
    results.insert("web".to_string(), web_status);
    results.insert("rss".to_string(), rss_status);
    results.insert("twitter".to_string(), twitter_status);

    let total_duration_ms = start.elapsed().as_millis() as u64;

    if json_output {
        let output = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "total_duration_ms": total_duration_ms,
            "channels": results,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("agent-reach doctor — health check\n");
        for (name, status) in &results {
            let icon = match status.status {
                agent_reach_core::doctor::Status::Ok => "✅",
                agent_reach_core::doctor::Status::Warning => "⚠️",
                agent_reach_core::doctor::Status::Unavailable => "❌",
            };
            println!("{} {}: {}", icon, name, status.message);
            println!("   duration: {}ms", status.duration_ms);
            for (backend, backend_status) in &status.backends {
                let b_icon = match backend_status {
                    BackendStatus::Available => "✓",
                    BackendStatus::RequiresConfig { .. } => "⚠",
                    BackendStatus::NotInstalled { .. } => "✗",
                    BackendStatus::Unavailable { .. } => "✗",
                };
                println!("     {} {} — {}", b_icon, backend, backend_status);
            }
            println!();
        }
        println!("Total: {}ms", total_duration_ms);
    }

    Ok(())
}
