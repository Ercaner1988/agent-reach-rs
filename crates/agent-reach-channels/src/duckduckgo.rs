//! DuckDuckGo channel — keyless web search via the HTML endpoint
//!
//! Backends:
//! 1. ddg-html (HTTP) — `html.duckduckgo.com/html/`, no key, no quota
//!
//! Why it exists: Exa's public MCP endpoint answers 429 under load, and a search
//! layer with exactly one engine has no answer to that. This is the second
//! keyless engine, so a rate limit degrades the result instead of ending it.
//!
//! It returns what a search engine returns — title and URL. It does **not**
//! filter to GitHub: a search channel that only speaks one site is not a search
//! channel, and callers that want repositories can say `site:github.com` in the
//! query like any other user of a search engine.

use agent_reach_core::{
    backend::{Backend, BackendStatus},
    channel::{Channel, ChannelOutput, ChannelResult},
    doctor::HealthStatus,
    Config, Error,
};
use async_trait::async_trait;
use regex::Regex;
use std::collections::HashMap;
use std::time::Instant;

/// DuckDuckGo HTML endpoint backend
pub struct DuckDuckGoHtmlBackend;

/// The HTML endpoint wraps every result URL in a redirect
/// (`/l/?uddg=<percent-encoded target>`), so the href has to be unwrapped
/// before it is worth anything. Title text follows on the same anchor.
fn parse_results(html: &str, limit: usize) -> Vec<serde_json::Value> {
    // (?s): the anchor and its title routinely straddle a newline in the real
    // response, and without it every wrapped result is silently dropped.
    let anchor = Regex::new(r#"(?s)<a[^>]+class="result__a"[^>]+href="([^"]+)"[^>]*>(.*?)</a>"#)
        .expect("valid regex");
    let tags = Regex::new(r"<[^>]+>").expect("valid regex");

    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for cap in anchor.captures_iter(html) {
        let href = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
        let url = unwrap_redirect(href);
        if url.is_empty() || !seen.insert(url.clone()) {
            continue;
        }
        let title = html_unescape(&tags.replace_all(cap.get(2).map_or("", |m| m.as_str()), ""));
        out.push(serde_json::json!({ "title": title.trim(), "url": url }));
        if out.len() >= limit {
            break;
        }
    }
    out
}

/// `//duckduckgo.com/l/?uddg=https%3A%2F%2F…&rut=…` → the real target.
fn unwrap_redirect(href: &str) -> String {
    let Some(rest) = href.split("uddg=").nth(1) else {
        // Not a redirect — take it as-is if it already looks absolute.
        return if href.starts_with("http") {
            href.to_string()
        } else {
            String::new()
        };
    };
    let encoded = rest.split('&').next().unwrap_or_default();
    urlencoding::decode(encoded)
        .map(|s| s.into_owned())
        .unwrap_or_default()
}

/// ponytail: the four entities DuckDuckGo actually emits in titles. A full
/// HTML-entity table is a dependency for no measured gain.
fn html_unescape(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
}

#[async_trait]
impl Backend for DuckDuckGoHtmlBackend {
    fn name(&self) -> &str {
        "ddg-html"
    }

    /// No key, no local binary. A probe here would cost a round trip on every
    /// call; `execute` reports a real failure if the endpoint is down.
    async fn is_available(&self, _config: &Config) -> BackendStatus {
        BackendStatus::Available
    }

    async fn execute(
        &self,
        action: &str,
        args: &[String],
        _config: &Config,
    ) -> agent_reach_core::backend::BackendResult<Vec<u8>> {
        if action != "search" {
            return Err(Error::UnsupportedAction("duckduckgo".into(), action.into()));
        }
        let query = args.first().ok_or_else(|| {
            Error::BackendExecution(self.name().into(), "Missing search query argument".into())
        })?;
        let limit = args
            .get(1)
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(10);

        let url = format!(
            "https://html.duckduckgo.com/html/?q={}",
            urlencoding::encode(query)
        );
        // An honest user-agent is served normally; the endpoint does not gate on
        // it. What it does gate on is burst rate, and it answers a burst with
        // `202` and an interstitial that carries no results — measured, not
        // assumed. One paced retry clears it. We do not disguise the client.
        let client = reqwest::Client::builder()
            .user_agent(concat!("agent-reach/", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| Error::Network(e.to_string()))?;

        let mut html = String::new();
        let mut last_status = 0u16;
        for attempt in 0..3 {
            if attempt > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(3 * attempt)).await;
            }
            let response = client
                .get(&url)
                .send()
                .await
                .map_err(|e| Error::Network(e.to_string()))?;
            let status = response.status();
            last_status = status.as_u16();
            if !status.is_success() {
                return Err(Error::BackendExecution(
                    self.name().into(),
                    format!("HTTP {status}"),
                ));
            }
            html = response
                .text()
                .await
                .map_err(|e| Error::Network(e.to_string()))?;
            // 202 is the throttle page; so is a 200 body with no result markup.
            if last_status != 202 && html.contains("result__a") {
                break;
            }
        }

        let results = parse_results(&html, limit);
        if results.is_empty() {
            // Say which of the two it was. "Layout changed" and "we were
            // throttled" need opposite responses, and a message that cannot
            // tell them apart sends the next reader after the wrong one.
            return Err(Error::BackendExecution(
                self.name().into(),
                format!(
                    "no results (HTTP {last_status}, {} bytes, result markup {})",
                    html.len(),
                    if html.contains("result__a") {
                        "present but unparsed — layout changed"
                    } else {
                        "absent — throttled or blocked"
                    }
                ),
            ));
        }
        serde_json::to_vec(&serde_json::json!({ "results": results }))
            .map_err(|e| Error::Decode(e.to_string()))
    }
}

/// DuckDuckGo channel orchestrator
pub struct DuckDuckGoChannel {
    backends: Vec<Box<dyn Backend>>,
}

impl DuckDuckGoChannel {
    pub fn new() -> Self {
        Self {
            backends: vec![Box::new(DuckDuckGoHtmlBackend)],
        }
    }
}

impl Default for DuckDuckGoChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Channel for DuckDuckGoChannel {
    fn platform(&self) -> &str {
        "duckduckgo"
    }

    fn actions(&self) -> Vec<String> {
        vec!["search".into()]
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

    #[test]
    fn test_unwrap_redirect() {
        assert_eq!(
            unwrap_redirect(
                "//duckduckgo.com/l/?uddg=https%3A%2F%2Fgithub.com%2Fexample-org%2Fexample-lib&rut=abc"
            ),
            "https://github.com/example-org/example-lib"
        );
        assert_eq!(
            unwrap_redirect("https://example.com"),
            "https://example.com"
        );
        assert_eq!(unwrap_redirect("/relative/path"), "");
    }

    #[test]
    fn test_parse_results() {
        // Shape taken from a live html.duckduckgo.com response.
        let html = r#"
          <a rel="nofollow" class="result__a" href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fgithub.com%2Fexample-org%2Fwidget&amp;rut=x">
            <b>widget</b> &amp; things</a>
          <a class="result__a" href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fdocs.rs%2Fwidget">docs.rs</a>
        "#;
        let hits = parse_results(html, 10);
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0]["url"], "https://github.com/example-org/widget");
        assert_eq!(hits[0]["title"], "widget & things");
    }

    #[test]
    fn test_parse_results_respects_limit() {
        let one = r#"<a class="result__a" href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fa.com">A</a>
                     <a class="result__a" href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fb.com">B</a>"#;
        assert_eq!(parse_results(one, 1).len(), 1);
    }
}
