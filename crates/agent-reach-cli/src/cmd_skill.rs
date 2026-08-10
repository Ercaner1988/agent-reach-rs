//! Skill subcommand — write/remove local Agent Reach skill guidance

use agent_reach_core::Config;
use anyhow::{bail, Result};
use std::path::PathBuf;

const SKILL_MD: &str = r#"---
name: agent-reach
description: Use Agent Reach to read web pages and RSS/Atom feeds from an AI agent.
---

# Agent Reach

Use `agent-reach execute --task-file <tasks.json>` for batch web/RSS reading.

Supported channels in this build:
- `web` with action `read`
- `rss` with actions `fetch` and `parse`

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
