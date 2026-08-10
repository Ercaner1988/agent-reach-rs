//! Install subcommand — safe environment setup for Agent Reach

use agent_reach_core::Config;
use anyhow::Result;

pub async fn install(
    env: String,
    proxy: Option<String>,
    safe: bool,
    dry_run: bool,
    channels: Option<String>,
) -> Result<()> {
    let config_path = Config::default_config_path()?;
    let config_dir = Config::config_dir()?;
    let selected_channels = channels.unwrap_or_else(|| "web,rss".to_string());

    println!("agent-reach install");
    println!("  environment: {}", env);
    println!("  config_dir: {}", config_dir.display());
    println!("  config_file: {}", config_path.display());
    println!("  channels: {}", selected_channels);

    if dry_run || safe {
        println!("  mode: {}", if dry_run { "dry-run" } else { "safe" });
        println!("  no filesystem changes were made");
        if proxy.is_some() {
            println!("  proxy would be saved to config");
        }
        return Ok(());
    }

    std::fs::create_dir_all(&config_dir)?;

    if let Some(proxy_value) = proxy {
        let mut config = Config::load().unwrap_or_default();
        config.proxy = Some(proxy_value);
        config.save()?;
        println!("  saved proxy setting");
    }

    println!("  ready: run `agent-reach doctor` to verify available channels");
    Ok(())
}
