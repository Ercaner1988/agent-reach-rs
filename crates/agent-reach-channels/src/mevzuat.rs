//! Mevzuat channel — Turkish legislation from mevzuat.gov.tr
//!
//! Backends:
//! 1. mevzuat-web (HTTP) — the public portal, no key and no account

use agent_reach_core::{
    backend::{require_payload, Backend, BackendStatus},
    channel::{Channel, ChannelOutput, ChannelResult},
    doctor::HealthStatus,
    Config, Error,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Instant;

/// A record is addressed by three numbers. Only the first is ever interesting to
/// a caller: `MevzuatTur` 1 is `Kanun` and `MevzuatTertip` 5 is the tertip every
/// law still in force belongs to, so both default and `["5237"]` is a whole query.
const VARSAYILAN_TUR: &str = "1";
const VARSAYILAN_TERTIP: &str = "5";

pub struct MevzuatWebBackend;

impl MevzuatWebBackend {
    /// `[no]`, `[no, tur]` or `[no, tur, tertip]`.
    fn kimlik(&self, args: &[String]) -> Result<(String, String, String), Error> {
        let no = args.first().ok_or_else(|| {
            Error::BackendExecution(self.name().into(), "Missing MevzuatNo argument".into())
        })?;
        let tur = args.get(1).map(String::as_str).unwrap_or(VARSAYILAN_TUR);
        let tertip = args.get(2).map(String::as_str).unwrap_or(VARSAYILAN_TERTIP);
        Ok((no.clone(), tur.into(), tertip.into()))
    }
}

#[async_trait]
impl Backend for MevzuatWebBackend {
    fn name(&self) -> &str {
        "mevzuat-web"
    }

    async fn is_available(&self, _config: &Config) -> BackendStatus {
        BackendStatus::Available
    }

    async fn execute(
        &self,
        action: &str,
        args: &[String],
        _config: &Config,
    ) -> agent_reach_core::backend::BackendResult<Vec<u8>> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| Error::Network(e.to_string()))?;
        let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

        let (no, tur, tertip) = self.kimlik(args)?;

        // Each action gets its own payload markers, verified against a captured
        // real record and a captured miss — see docs/channels/mevzuat.md.
        let (url, markers): (String, &[&str]) = match action {
            // The record page: publication dates and the links to the official
            // .doc/.pdf. Cheap, and the natural "does this law exist" probe.
            "mevzuat" => (
                format!(
                    "https://www.mevzuat.gov.tr/mevzuat?MevzuatNo={no}&MevzuatTur={tur}&MevzuatTertip={tertip}"
                ),
                &["MevzuatMetin", "MevzuatNo"],
            ),
            // The frame the record page embeds — this is where the articles
            // actually live. 700 KB for the penal code, versus 70 KB of shell.
            "metin" | "fihrist" => (
                format!(
                    "https://www.mevzuat.gov.tr/anasayfa/MevzuatFihristDetayIframe?MevzuatTur={tur}&MevzuatNo={no}&MevzuatTertip={tertip}"
                ),
                &["section-to-print", "Madde"],
            ),
            other => return Err(Error::UnsupportedAction("mevzuat".into(), other.into())),
        };

        let response = client
            .get(&url)
            .header("User-Agent", user_agent)
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Error::BackendExecution(
                self.name().into(),
                format!("HTTP {}", response.status()),
            ));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        // An unknown MevzuatNo does not 404. It 302s to /Anasayfa/ErrorPage?code=404,
        // which reqwest follows, so the backend is handed 200 and ~65 KB of error
        // page — indistinguishable from a hit by status alone (ADR 0004).
        require_payload(self.name(), &bytes, markers)?;

        Ok(bytes.to_vec())
    }
}

/// Mevzuat channel orchestrator
pub struct MevzuatChannel {
    backends: Vec<Box<dyn Backend>>,
}

impl MevzuatChannel {
    pub fn new() -> Self {
        Self {
            backends: vec![Box::new(MevzuatWebBackend)],
        }
    }
}

impl Default for MevzuatChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Channel for MevzuatChannel {
    fn platform(&self) -> &str {
        "mevzuat"
    }

    fn actions(&self) -> Vec<String> {
        vec!["mevzuat".into(), "metin".into(), "fihrist".into()]
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

        Err(last_error
            .unwrap_or_else(|| agent_reach_core::backend::unavailable(self.platform(), &skipped)))
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
    async fn test_mevzuat_web_availability() {
        let backend = MevzuatWebBackend;
        let config = Config::default();
        let status = backend.is_available(&config).await;
        assert!(matches!(status, BackendStatus::Available));
    }

    #[test]
    fn tur_ve_tertip_varsayilani_tek_argumanla_gelir() {
        let b = MevzuatWebBackend;
        let (no, tur, tertip) = b.kimlik(&["5237".to_string()]).unwrap();
        assert_eq!((no.as_str(), tur.as_str(), tertip.as_str()), ("5237", "1", "5"));
    }

    #[test]
    fn verilen_tur_ve_tertip_varsayilani_ezer() {
        let b = MevzuatWebBackend;
        let args = ["5210".to_string(), "21".to_string(), "5".to_string()];
        let (no, tur, tertip) = b.kimlik(&args).unwrap();
        assert_eq!((no.as_str(), tur.as_str(), tertip.as_str()), ("5210", "21", "5"));
    }

    #[test]
    fn mevzuat_no_olmadan_calismaz() {
        assert!(MevzuatWebBackend.kimlik(&[]).is_err());
    }
}
