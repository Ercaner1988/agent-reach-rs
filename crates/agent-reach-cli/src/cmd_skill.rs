//! Skill subcommand — write/remove local Agent Reach skill guidance

use agent_reach_core::Config;
use anyhow::{bail, Result};
use std::path::PathBuf;

const SKILL_MD: &str = r#"---
name: agent-reach
description: Use Agent Reach to read web pages, feeds, and social platforms from an AI agent.
---

# Agent Reach

Use `agent-reach execute --task-file <tasks.json>` for batch reading tasks.

Supported channels in this build (channel — actions):
- `web` — `read`
- `rss` — `fetch`, `parse`
- `twitter` — `search`, `timeline`, `thread`
- `youtube` — `metadata`, `transcript`, `search`
- `github` — `repo`, `issue`, `pr`, `search`
- `reddit` — `subreddit`, `search`, `post`
- `bilibili` — `video`, `info`, `search`
- `xiaohongshu` — `note`, `search`
- `linkedin` — `profile`, `company`, `search`
- `v2ex` — `topic`, `node`, `hot`, `latest`
- `xueqiu` — `quote`, `stock`, `timeline`, `search`
- `xiaoyuzhou` — `podcast`, `episode`
- `exa` — `search`
- `duckduckgo` — `search`
- `turath` — `search`, `book`, `author`, `page`

Some channels need credentials in `~/.agent-reach/config.yaml`
(see `agent-reach configure` and `agent-reach doctor`).

Health check:
```bash
agent-reach doctor
```
"#;

pub async fn skill(install: bool, uninstall: bool) -> Result<()> {
    if install == uninstall {
        bail!("choose exactly one: --install or --uninstall");
    }

    let skill_path = local_skill_path()?;

    if install {
        if let Some(parent) = skill_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&skill_path, SKILL_MD)?;
        println!("installed local skill file: {}", skill_path.display());
    } else {
        if skill_path.exists() {
            std::fs::remove_file(&skill_path)?;
            println!("removed local skill file: {}", skill_path.display());
        } else {
            println!("local skill file was not present: {}", skill_path.display());
        }
    }

    Ok(())
}

fn local_skill_path() -> Result<PathBuf> {
    Ok(Config::config_dir()?
        .join("skills")
        .join("agent-reach")
        .join("SKILL.md"))
}
