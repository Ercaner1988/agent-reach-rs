//! Doctor subcommand — check platform availability and health

use agent_reach_channels::{
    BilibiliChannel, DuckDuckGoChannel, ExaChannel, GitHubChannel, LinkedinChannel, RedditChannel,
    RssChannel, TwitterChannel, V2exChannel, WebChannel, XiaohongshuChannel, XiaoyuzhouChannel,
    XueqiuChannel, YouTubeChannel,
};
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
    let youtube_status = YouTubeChannel::new().health_check(&config).await;
    let github_status = GitHubChannel::new().health_check(&config).await;
    let reddit_status = RedditChannel::new().health_check(&config).await;
    let bilibili_status = BilibiliChannel::new().health_check(&config).await;
    let xiaohongshu_status = XiaohongshuChannel::new().health_check(&config).await;
    let linkedin_status = LinkedinChannel::new().health_check(&config).await;
    let v2ex_status = V2exChannel::new().health_check(&config).await;
    let xueqiu_status = XueqiuChannel::new().health_check(&config).await;
    let xiaoyuzhou_status = XiaoyuzhouChannel::new().health_check(&config).await;
    let exa_status = ExaChannel::new().health_check(&config).await;
    let duckduckgo_status = DuckDuckGoChannel::new().health_check(&config).await;

    let mut results = HashMap::new();
    results.insert("web".to_string(), web_status);
    results.insert("rss".to_string(), rss_status);
    results.insert("twitter".to_string(), twitter_status);
    results.insert("youtube".to_string(), youtube_status);
    results.insert("github".to_string(), github_status);
    results.insert("reddit".to_string(), reddit_status);
    results.insert("bilibili".to_string(), bilibili_status);
    results.insert("xiaohongshu".to_string(), xiaohongshu_status);
    results.insert("linkedin".to_string(), linkedin_status);
    results.insert("v2ex".to_string(), v2ex_status);
    results.insert("xueqiu".to_string(), xueqiu_status);
    results.insert("xiaoyuzhou".to_string(), xiaoyuzhou_status);
    results.insert("exa".to_string(), exa_status);
    results.insert("duckduckgo".to_string(), duckduckgo_status);

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
