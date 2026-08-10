//! Configure subcommand — manage channel settings

use agent_reach_core::Config;
use anyhow::{bail, Result};

pub async fn configure(
    key: Option<String>,
    value: Option<String>,
    unset: bool,
    json: bool,
    from_browser: Option<String>,
) -> Result<()> {
    if let Some(browser) = from_browser {
        bail!(
            "browser cookie extraction is not implemented yet for '{}'; no configuration was changed",
            browser
        );
    }
    if unset && value.is_some() {
        bail!("--unset cannot be combined with a value");
    }

    match (key, value) {
        (Some(k), Some(v)) => {
            let mut config = Config::load().unwrap_or_default();
            config.set(&k, v);
            config.save()?;
            let path = Config::default_config_path()?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({"key": k, "action": "set", "path": path})
                );
            } else {
                println!("saved {} to {}", k, path.display());
            }
        }
        (Some(k), None) => {
            let mut config = Config::load().unwrap_or_default();
            if unset {
                if !config.unset(&k) {
                    bail!("config key '{}' is not set", k);
                }
                config.save()?;
                if json {
                    println!("{}", serde_json::json!({"key": k, "action": "unset"}));
                } else {
                    println!("unset {}", k);
                }
            } else if json {
                println!("{}", serde_json::json!({"key": k, "value": config.get(&k)}));
            } else {
                match config.get(&k) {
                    Some(v) => println!("{}={}", k, v),
                    None => bail!("config key '{}' is not set", k),
                }
            }
        }
        (None, None) => {
            if unset {
                bail!("--unset requires a config key");
            }
            let config = Config::load().unwrap_or_default();
            if json {
                println!("{}", serde_json::to_string_pretty(&config)?);
            } else {
                println!("{}", serde_yaml::to_string(&config)?);
            }
        }
        (None, Some(_)) => bail!("value was provided without a config key"),
    }

    Ok(())
}
