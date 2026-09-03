//! UIN Jakarta channel — the open-access journal portal at `journal.uinjkt.ac.id`
//!
//! Backends:
//! 1. uinjkt-oai (HTTP) — OAI-PMH 2.0, no key and no account
//!
//! The portal is Open Journal Systems, and OJS speaks OAI-PMH out of the box:
//! one site-wide endpoint (`/index.php/index/oai`) that lists every journal as
//! a set, plus one endpoint per journal (`/index.php/<dergi>/oai`). Measured
//! 2026-09-03, all live:
//!
//! ```text
//! Identify                              → 200  (repositoryName per journal)
//! ListSets                              → 200  (100 sets/page, resumptionToken)
//! ListRecords&set=ahkam                 → 200  (100 records/page, oai_dc)
//! ListRecords&set=iqtishad              → 200
//! /index.php/tauhidinomics/oai Identify → 200
//! /index.php/ahkam/article/view/929     → 200  (full text, no login)
//! ```
//!
//! `robots.txt` bars only the back-office paths (`/admin/`, `/submission/` …);
//! the OAI endpoint and article pages are not disallowed. The target journals
//! for Islamic economics and law are sets: `iqtishad` (Al-Iqtishad), `ahkam`
//! (AHKAM: Jurnal Ilmu Syariah), `tauhidinomics` (Tauhidinomics: Journal of
//! Islamic Banking and Economics).
//!
//! OAI-PMH has no full-text search verb — it is a harvesting protocol: you
//! list sets, list records in a set, or get one record by identifier. Search
//! across the portal belongs to a later backend (the OJS search page or an
//! external index); what this channel guarantees is complete, paginated,
//! standards-based metadata for every article, with DOIs.

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

const OAI: &str = "https://journal.uinjkt.ac.id/index.php/index/oai";

/// UIN Jakarta OAI-PMH backend.
pub struct UinjktOaiBackend;

impl UinjktOaiBackend {
    /// The URL an action reaches, and the cassette key that stands for it.
    ///
    /// Separated from the request so the routing can be tested without a
    /// network: every argument mistake shows up here.
    fn route(action: &str, args: &[String]) -> Result<(String, Vec<String>), Error> {
        let arg = |i: usize, what: &str| -> Result<&String, Error> {
            args.get(i).ok_or_else(|| {
                Error::BackendExecution("uinjkt-oai".into(), format!("Missing {what} argument"))
            })
        };

        match action {
            // Repository identity — the cheapest possible liveness probe.
            "identify" => Ok((
                format!("{OAI}?verb=Identify"),
                vec!["identify".into()],
            )),
            // Every journal and issue as a set. A resumptionToken continues the
            // list; the portal pages at 100 sets.
            "journals" => {
                let token = args.first();
                let url = match token {
                    Some(t) => format!(
                        "{OAI}?verb=ListSets&resumptionToken={}",
                        urlencoding::encode(t)
                    ),
                    None => format!("{OAI}?verb=ListSets"),
                };
                Ok((url, vec!["journals".into(), token.cloned().unwrap_or_default()]))
            }
            // Articles of one journal set, Dublin Core metadata, 100 per page.
            // `set` is the journal's setSpec: `ahkam`, `iqtishad`,
            // `tauhidinomics`, … A resumptionToken continues the harvest.
            "articles" => {
                let set = arg(0, "journal set")?;
                let url = match args.get(1) {
                    Some(token) => format!(
                        "{OAI}?verb=ListRecords&resumptionToken={}",
                        urlencoding::encode(token)
                    ),
                    None => format!(
                        "{OAI}?verb=ListRecords&metadataPrefix=oai_dc&set={}",
                        urlencoding::encode(set)
                    ),
                };
                Ok((
                    url,
                    vec![
                        "articles".into(),
                        set.clone(),
                        args.get(1).cloned().unwrap_or_default(),
                    ],
                ))
            }
            // One article by its OAI identifier, e.g.
            // `oai:journal.uinjkt.ac.id:article/929`.
            "record" => {
                let id = arg(0, "OAI identifier")?;
                Ok((
                    format!(
                        "{OAI}?verb=GetRecord&metadataPrefix=oai_dc&identifier={}",
                        urlencoding::encode(id)
                    ),
                    vec!["record".into(), id.clone()],
                ))
            }
            other => Err(Error::UnsupportedAction("uinjkt".into(), other.into())),
        }
    }
}

#[async_trait]
impl Backend for UinjktOaiBackend {
    fn name(&self) -> &str {
        "uinjkt-oai"
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

        let mut parts = vec!["uinjkt"];
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

/// The value of one XML element, unescaped enough for Dublin Core text.
fn tag_value<'a>(block: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = block.find(&open)? + open.len();
    let end = block[start..].find(&close)? + start;
    Some(block[start..end].trim())
}

/// Every value of a repeatable XML element (`dc:identifier`, `dc:creator` …).
fn tag_values<'a>(block: &'a str, tag: &str) -> Vec<&'a str> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let mut out = Vec::new();
    let mut rest = block;
    while let Some(s) = rest.find(&open) {
        let from = s + open.len();
        let Some(e) = rest[from..].find(&close) else { break };
        out.push(rest[from..from + e].trim());
        rest = &rest[from + e + close.len()..];
    }
    out
}

/// Lift the OAI-PMH XML into JSON the caller can index.
///
/// The endpoint answers XML; callers of this channel work in JSON. Each
/// `<record>` becomes an object with its header (`identifier`, `datestamp`,
/// `sets`) and the Dublin Core fields that name an article (`title`,
/// `creators`, `date`, `identifiers` — one of which is the DOI —
/// `description`). The portal's `resumptionToken` is lifted alongside, so the
/// next page is one argument away instead of buried in markup.
///
/// A response that matches none of this — an OAI error envelope, say — is
/// passed through as raw text: dropping data because it surprised us is how a
/// reader ends up trusting a shorter list.
fn oai_to_json(xml: &str) -> serde_json::Value {
    let mut records = Vec::new();
    let mut rest = xml;
    while let Some(s) = rest.find("<record>") {
        let Some(e) = rest[s..].find("</record>") else { break };
        let block = &rest[s..s + e];
        records.push(serde_json::json!({
            "identifier": tag_value(block, "identifier"),
            "datestamp": tag_value(block, "datestamp"),
            "sets": tag_values(block, "setSpec"),
            "title": tag_value(block, "dc:title"),
            "creators": tag_values(block, "dc:creator"),
            "subjects": tag_values(block, "dc:subject"),
            "date": tag_value(block, "dc:date"),
            "identifiers": tag_values(block, "dc:identifier"),
            "description": tag_value(block, "dc:description"),
        }));
        rest = &rest[s + e + "</record>".len()..];
    }

    let token = tag_value(xml, "resumptionToken").filter(|t| !t.is_empty());

    if records.is_empty() && token.is_none() {
        return serde_json::json!({ "text": xml });
    }

    let mut out = serde_json::json!({ "records": records });
    if let Some(t) = token {
        out["resumptionToken"] = serde_json::Value::String(t.to_string());
    }
    out
}

/// UIN Jakarta channel orchestrator.
pub struct UinjktChannel {
    backends: Vec<Box<dyn Backend>>,
}

impl UinjktChannel {
    pub fn new() -> Self {
        Self {
            backends: vec![Box::new(UinjktOaiBackend)],
        }
    }
}

impl Default for UinjktChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Channel for UinjktChannel {
    fn platform(&self) -> &str {
        "uinjkt"
    }

    fn actions(&self) -> Vec<String> {
        vec![
            "identify".into(),
            "journals".into(),
            "articles".into(),
            "record".into(),
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
                    let body = String::from_utf8_lossy(&data);
                    let json_data = oai_to_json(&body);
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
    fn articles_of_a_journal_are_addressed_by_its_set() {
        let (url, key) = UinjktOaiBackend::route("articles", &args(&["iqtishad"])).unwrap();
        assert_eq!(
            url,
            "https://journal.uinjkt.ac.id/index.php/index/oai?verb=ListRecords&metadataPrefix=oai_dc&set=iqtishad"
        );
        assert_eq!(key[1], "iqtishad");
    }

    #[test]
    fn a_resumption_token_continues_the_harvest() {
        let (url, _) =
            UinjktOaiBackend::route("articles", &args(&["ahkam", "TOKEN/123"]).unwrap()[..])
                .unwrap();
        assert!(url.contains("verb=ListRecords&resumptionToken=TOKEN%2F123"), "{url}");
        // A token replaces the set: OAI-PMH forbids mixing them.
        assert!(!url.contains("set="), "{url}");
    }

    #[test]
    fn a_missing_argument_is_named_rather_than_guessed() {
        let err = UinjktOaiBackend::route("articles", &[]).unwrap_err();
        assert!(err.to_string().contains("journal set"), "got: {err}");
        assert!(UinjktOaiBackend::route("record", &[]).is_err());
    }

    #[test]
    fn an_unknown_action_is_rejected() {
        assert!(UinjktOaiBackend::route("search", &args(&["x"])).is_err());
    }

    #[test]
    fn records_are_lifted_out_of_the_xml_with_their_doi() {
        let xml = r#"<?xml version="1.0"?>
<OAI-PMH><ListRecords>
<record><header><identifier>oai:journal.uinjkt.ac.id:article/929</identifier>
<datestamp>2020-01-01</datestamp><setSpec>ahkam</setSpec></header>
<metadata><oai_dc:dc>
<dc:title>Islamic Law in Indonesia</dc:title>
<dc:creator>Doe, John</dc:creator>
<dc:date>2013-12-01</dc:date>
<dc:identifier>https://journal.uinjkt.ac.id/index.php/ahkam/article/view/929</dc:identifier>
<dc:identifier>10.15408/ajis.v13i2.929</dc:identifier>
</oai_dc:dc></metadata></record>
<resumptionToken>NEXT</resumptionToken>
</ListRecords></OAI-PMH>"#;
        let out = oai_to_json(xml);
        assert_eq!(out["records"][0]["title"], "Islamic Law in Indonesia");
        assert_eq!(out["records"][0]["identifiers"][1], "10.15408/ajis.v13i2.929");
        assert_eq!(out["records"][0]["sets"][0], "ahkam");
        assert_eq!(out["resumptionToken"], "NEXT");
    }

    #[test]
    fn an_unexpected_envelope_survives_untouched() {
        let out = oai_to_json("<OAI-PMH><error code=\"badVerb\">nope</error></OAI-PMH>");
        assert!(out["text"].as_str().unwrap().contains("badVerb"));
    }

    #[tokio::test]
    async fn the_backend_needs_no_key_and_no_account() {
        let status = UinjktOaiBackend.is_available(&Config::default()).await;
        assert!(matches!(status, BackendStatus::Available));
    }
}
