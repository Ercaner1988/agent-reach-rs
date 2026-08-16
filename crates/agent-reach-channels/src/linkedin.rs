//! LinkedIn channel — profiles, companies, posts
//!
//! Backends:
//! 1. linkedin-api (Python subprocess) — requires linkedin_username, linkedin_password

use agent_reach_core::{
    backend::{Backend, BackendStatus},
    channel::{Channel, ChannelOutput, ChannelResult},
    doctor::HealthStatus,
    Config, Error,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Instant;

/// LinkedIn API backend (Python subprocess via linkedin-api package)
pub struct LinkedinApiBackend;

#[async_trait]
impl Backend for LinkedinApiBackend {
    fn name(&self) -> &str {
        "linkedin-api"
    }

    async fn is_available(&self, config: &Config) -> BackendStatus {
        let check = tokio::process::Command::new("python3")
            .arg("-c")
            .arg("import linkedin_api")
            .output()
            .await;

        if check.is_err() || !check.unwrap().status.success() {
            return BackendStatus::NotInstalled {
                command: "python3 -m pip install linkedin-api".into(),
            };
        }

        let mut missing = Vec::new();
        if config.linkedin_username.is_none() {
            missing.push("linkedin_username".into());
        }
        if config.linkedin_password.is_none() {
            missing.push("linkedin_password".into());
        }

        if missing.is_empty() {
            BackendStatus::Available
        } else {
            BackendStatus::RequiresConfig { missing }
        }
    }

    async fn execute(
        &self,
        action: &str,
        args: &[String],
        config: &Config,
    ) -> agent_reach_core::backend::BackendResult<Vec<u8>> {
        let username = config
            .linkedin_username
            .as_ref()
            .ok_or_else(|| Error::Config("linkedin_username not set".into()))?;
        let password = config
            .linkedin_password
            .as_ref()
            .ok_or_else(|| Error::Config("linkedin_password not set".into()))?;

        let script = match action {
            "profile" => {
                let public_id = args.first().ok_or_else(|| {
                    Error::BackendExecution(
                        self.name().into(),
                        "Missing profile public_id argument".into(),
                    )
                })?;
                format!(
                    r#"
import json
from linkedin_api import Linkedin

api = Linkedin('{}', '{}')
profile = api.get_profile('{}')
print(json.dumps(profile))
"#,
                    username, password, public_id
                )
            }
            "company" => {
                let company_id = args.first().ok_or_else(|| {
                    Error::BackendExecution(
                        self.name().into(),
                        "Missing company public_id argument".into(),
                    )
                })?;
                format!(
                    r#"
import json
from linkedin_api import Linkedin

api = Linkedin('{}', '{}')
company = api.get_company('{}')
print(json.dumps(company))
"#,
                    username, password, company_id
                )
            }
            "search" => {
                let query = args.first().ok_or_else(|| {
                    Error::BackendExecution(self.name().into(), "Missing query argument".into())
                })?;
                format!(
                    r#"
import json
from linkedin_api import Linkedin

api = Linkedin('{}', '{}')
results = api.search_people(keyword_title='{}', limit=5)
print(json.dumps(results))
"#,
                    username, password, query
                )
            }
            other => {
                return Err(Error::UnsupportedAction("linkedin".into(), other.into()));
            }
        };

        let output = tokio::process::Command::new("python3")
            .arg("-c")
            .arg(&script)
            .output()
            .await
            .map_err(|e| Error::BackendExecution(self.name().into(), e.to_string()))?;

        if !output.status.success() {
            return Err(Error::BackendExecution(
                self.name().into(),
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(output.stdout)
    }
}

/// LinkedIn Channel orchestrator
pub struct LinkedinChannel {
    backends: Vec<Box<dyn Backend>>,
}

impl LinkedinChannel {
    pub fn new() -> Self {
        Self {
            backends: vec![Box::new(LinkedinApiBackend)],
        }
    }
}

impl Default for LinkedinChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Channel for LinkedinChannel {
    fn platform(&self) -> &str {
        "linkedin"
    }

    fn actions(&self) -> Vec<String> {
        vec!["profile".into(), "company".into(), "search".into()]
    }

    async fn execute(
        &self,
        action: &str,
        args: &[String],
        config: &Config,
    ) -> ChannelResult<ChannelOutput> {
        let start = Instant::now();

        let mut last_error = None;
        let mut skipped = Vec::new();
        for backend in &self.backends {
            let status = backend.is_available(config).await;
            if !matches!(status, BackendStatus::Available) {
                tracing::debug!("Backend {} not available: {}", backend.name(), status);
                skipped.push((backend.name().to_string(), status));
                continue;
            }

            match backend.execute(action, args, config).await {
                Ok(data) => {
                    let json_data: serde_json::Value = serde_json::from_slice(&data)
                        .unwrap_or_else(
                            |_| serde_json::json!({ "text": String::from_utf8_lossy(&data) }),
                        );
                    return Ok(ChannelOutput {
                        platform: self.platform().into(),
                        action: action.into(),
                        backend: backend.name().into(),
                        data: json_data,
                        duration_ms: start.elapsed().as_millis() as u64,
                    });
                }
                Err(e) => {
                    tracing::warn!("Backend {} failed: {}", backend.name(), e);
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            agent_reach_core::backend::unavailable(self.platform(), &skipped)
        }))
    }

    async fn health_check(&self, config: &Config) -> HealthStatus {
        let start = Instant::now();
        let mut backends_status = HashMap::new();

        for backend in &self.backends {
            let status = backend.is_available(config).await;
            backends_status.insert(backend.name().into(), status);
        }

        HealthStatus::new(
            self.platform().into(),
            backends_status,
            start.elapsed().as_millis() as u64,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_linkedin_api_requires_credentials() {
        let backend = LinkedinApiBackend;
        let config = Config::default();
        let status = backend.is_available(&config).await;

        // tests run without `linkedin-api` package installed will return NotInstalled.
        // We only assert it requires config if it passes the install check.
        match status {
            BackendStatus::RequiresConfig { .. } => {}
            BackendStatus::NotInstalled { .. } => {}
            _ => panic!("Expected RequiresConfig or NotInstalled, got {:?}", status),
        }
    }
}
