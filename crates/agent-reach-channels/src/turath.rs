//! Turath channel — the classical Arabic library behind `app.turath.io`
//!
//! Backends:
//! 1. turath-api (HTTP) — `api.turath.io`, no key and no account
//!
//! The site is a single-page app with no API documentation anywhere; the four
//! endpoints below were found by probing, and the bare host answering
//! `400 {"error": true}` is what showed there was an API there at all.
//!
//! There is no `robots.txt` either — every path returns the app's index page —
//! so the site states neither a ban nor a permission. One request per call, an
//! honest User-Agent, and the cassette in front of the network is the whole
//! pacing policy. Finding a door open is not a reason to run through it.

use agent_reach_core::{
    backend::{Backend, BackendStatus},
    cassette,
    channel::{Channel, ChannelOutput, ChannelResult},
    doctor::HealthStatus,
    Config, Error,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Instant;

const BASE: &str = "https://api.turath.io";

/// Turath REST backend.
pub struct TurathApiBackend;

impl TurathApiBackend {
    /// The URL an action reaches, and the cassette key that stands for it.
    ///
    /// Separated from the request so the routing can be tested without a
    /// network: every argument mistake shows up here.
    fn route(action: &str, args: &[String]) -> Result<(String, Vec<String>), Error> {
        let arg = |i: usize, what: &str| -> Result<&String, Error> {
            args.get(i).ok_or_else(|| {
                Error::BackendExecution("turath-api".into(), format!("Missing {what} argument"))
            })
        };

        match action {
            // `page` is the result page, not a page of a book — the API counts
            // twenty records to a page.
            "search" => {
                let query = arg(0, "query")?;
                let page = args.get(1).map(String::as_str).unwrap_or("1");
                Ok((
                    format!(
                        "{BASE}/search?q={}&page={}",
                        urlencoding::encode(query),
                        urlencoding::encode(page)
                    ),
                    vec!["search".into(), query.clone(), page.into()],
                ))
            }
            "book" => {
                let id = arg(0, "book id")?;
                Ok((
                    format!("{BASE}/book?id={}", urlencoding::encode(id)),
                    vec!["book".into(), id.clone()],
                ))
            }
            "author" => {
                let id = arg(0, "author id")?;
                Ok((
                    format!("{BASE}/author?id={}", urlencoding::encode(id)),
                    vec!["author".into(), id.clone()],
                ))
            }
            // `pg` is the printed page number, the one `meta.page` carries.
            // `page_id` looks like it should work here and does not: the API
            // answers 200 with an empty object, which is worse than an error.
            "page" => {
                let book_id = arg(0, "book id")?;
                let pg = arg(1, "page number")?;
                Ok((
                    format!(
                        "{BASE}/page?book_id={}&pg={}",
                        urlencoding::encode(book_id),
                        urlencoding::encode(pg)
                    ),
                    vec!["page".into(), book_id.clone(), pg.clone()],
                ))
            }
            other => Err(Error::UnsupportedAction("turath".into(), other.into())),
        }
    }
}

#[async_trait]
impl Backend for TurathApiBackend {
    fn name(&self) -> &str {
        "turath-api"
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
        let (url, key_parts) = Self::route(action, args)?;

        let mut parts = vec!["turath"];
        parts.extend(key_parts.iter().map(String::as_str));
        let tape_key = cassette::key(&parts);
        if let Some(rec) = cassette::load(&tape_key) {
            return if rec.status == 200 {
                Ok(rec.body.into_bytes())
            } else {
                Err(Error::BackendExecution(
                    self.name().into(),
                    format!("HTTP {} (replayed)", rec.status),
                ))
            };
        }

        let response = reqwest::Client::new()
            .get(&url)
            .header(
                "User-Agent",
                "agent-reach-rs/0.1 (+https://github.com/Ercaner1988/agent-reach-rs)",
            )
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        // Refusals are recorded with their code, so the "asked but not answered"
        // path can be replayed instead of waited for.
        cassette::save(
            &tape_key,
            &cassette::Recording {
                status: status.as_u16(),
                body: body.clone(),
            },
        );

        if !status.is_success() {
            return Err(Error::BackendExecution(
                self.name().into(),
                format!("HTTP {status}"),
            ));
        }

        Ok(body.into_bytes())
    }
}

/// Lift `meta` out of the string it arrives as.
///
/// Every search hit carries `meta` as *JSON inside a JSON string*: the volume,
/// the printed page and the book and author names are all in there, and a
/// caller that does not know to parse it twice sees an opaque blob. The volume
/// and page are the reason to use this source at all — they are what a citation
/// needs — so the channel unpacks them rather than passing the wart along.
///
/// A hit whose `meta` will not parse is left exactly as it came: dropping data
/// because it surprised us is how a reader ends up trusting a shorter list.
fn unwrap_meta(mut value: serde_json::Value) -> serde_json::Value {
    let Some(hits) = value.get_mut("data").and_then(|d| d.as_array_mut()) else {
        return value;
    };
    for hit in hits.iter_mut() {
        let parsed = hit
            .get("meta")
            .and_then(|m| m.as_str())
            .and_then(|m| serde_json::from_str::<serde_json::Value>(m).ok());
        if let Some(parsed) = parsed {
            if let Some(obj) = hit.as_object_mut() {
                obj.insert("meta".into(), parsed);
            }
        }
    }
    value
}

/// Turath channel orchestrator.
pub struct TurathChannel {
    backends: Vec<Box<dyn Backend>>,
}

impl TurathChannel {
    pub fn new() -> Self {
        Self {
            backends: vec![Box::new(TurathApiBackend)],
        }
    }
}

impl Default for TurathChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Channel for TurathChannel {
    fn platform(&self) -> &str {
        "turath"
    }

    fn actions(&self) -> Vec<String> {
        vec![
            "search".into(),
            "book".into(),
            "author".into(),
            "page".into(),
        ]
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
                        .map(unwrap_meta)
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

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn search_defaults_to_the_first_page_and_encodes_the_query() {
        let (url, key) = TurathApiBackend::route("search", &args(&["الفقه الإسلامي"])).unwrap();
        assert!(url.starts_with("https://api.turath.io/search?q="));
        assert!(
            url.ends_with("&page=1"),
            "missing page defaults to 1: {url}"
        );
        assert!(!url.contains(' '), "the space must be encoded: {url}");
        assert_eq!(key[0], "search");
    }

    #[test]
    fn a_page_is_addressed_by_its_printed_number() {
        let (url, _) = TurathApiBackend::route("page", &args(&["4473", "155"])).unwrap();
        // `pg`, not `page_id`: page_id answers 200 with an empty body, so the
        // wrong one fails silently rather than loudly.
        assert_eq!(url, "https://api.turath.io/page?book_id=4473&pg=155");
    }

    #[test]
    fn a_missing_argument_is_named_rather_than_guessed() {
        let err = TurathApiBackend::route("page", &args(&["4473"])).unwrap_err();
        assert!(err.to_string().contains("page number"), "got: {err}");
        assert!(TurathApiBackend::route("search", &[]).is_err());
    }

    #[test]
    fn an_unknown_action_is_rejected() {
        assert!(TurathApiBackend::route("browse", &args(&["x"])).is_err());
    }

    #[test]
    fn meta_is_unpacked_so_a_citation_can_read_the_volume_and_page() {
        let raw = serde_json::json!({
            "count": 2,
            "data": [
                { "book_id": 4473, "meta": "{\"vol\":\"1\",\"page\":155,\"book_name\":\"كتاب\"}" },
                { "book_id": 9, "meta": "not json at all" }
            ]
        });
        let out = unwrap_meta(raw);
        assert_eq!(out["data"][0]["meta"]["page"], 155);
        assert_eq!(out["data"][0]["meta"]["vol"], "1");
        // The one that could not be parsed survives untouched.
        assert_eq!(out["data"][1]["meta"], "not json at all");
    }

    #[tokio::test]
    async fn the_backend_needs_no_key_and_no_account() {
        let status = TurathApiBackend.is_available(&Config::default()).await;
        assert!(matches!(status, BackendStatus::Available));
    }
}
