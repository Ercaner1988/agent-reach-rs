//! RSS/Atom channel — fetch and parse feed URLs

use agent_reach_core::{
    backend::{Backend, BackendStatus},
    channel::{Channel, ChannelOutput, ChannelResult},
    doctor::HealthStatus,
    Config, Error,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Instant;

/// RSS parser backend — parses RSS 2.0 and Atom XML
pub struct RssParserBackend;

#[async_trait]
impl Backend for RssParserBackend {
    fn name(&self) -> &str {
        "rss-parser"
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
        if action != "parse" {
            return Err(Error::UnsupportedAction("rss".into(), action.into()));
        }

        let xml = args.first().ok_or_else(|| {
            Error::BackendExecution(self.name().into(), "Missing XML argument".into())
        })?;

        // Try RSS 2.0 first, then Atom
        let parsed = parse_feed(xml).map_err(|e| {
            Error::BackendExecution(self.name().into(), format!("Parse failed: {}", e))
        })?;

        let json = serde_json::to_vec(&parsed).map_err(|e| {
            Error::BackendExecution(self.name().into(), format!("JSON serialize failed: {}", e))
        })?;

        Ok(json)
    }
}

/// RSS channel — fetch and parse feed URLs
pub struct RssChannel {
    backends: Vec<Box<dyn Backend>>,
}

impl RssChannel {
    pub fn new() -> Self {
        Self {
            backends: vec![Box::new(RssParserBackend)],
        }
    }
}

impl Default for RssChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Channel for RssChannel {
    fn platform(&self) -> &str {
        "rss"
    }

    fn actions(&self) -> Vec<String> {
        vec!["fetch".into(), "parse".into()]
    }

    async fn execute(
        &self,
        action: &str,
        args: &[String],
        config: &Config,
    ) -> ChannelResult<ChannelOutput> {
        let start = Instant::now();

        // Fetch feed content from URL (if action is "fetch"), then parse
        let xml = match action {
            "fetch" => {
                let url = args.first().ok_or_else(|| {
                    Error::BackendExecution(self.platform().into(), "Missing URL argument".into())
                })?;
                fetch_url(url, config).await?
            }
            "parse" => args.first().cloned().ok_or_else(|| {
                Error::BackendExecution(self.platform().into(), "Missing XML argument".into())
            })?,
            _ => {
                return Err(Error::UnsupportedAction(
                    self.platform().into(),
                    action.into(),
                ))
            }
        };

        // Parse
        for backend in &self.backends {
            let status = backend.is_available(config).await;
            if !matches!(status, BackendStatus::Available) {
                tracing::debug!("Backend {} not available: {}", backend.name(), status);
                continue;
            }

            match backend
                .execute("parse", std::slice::from_ref(&xml), config)
                .await
            {
                Ok(data) => {
                    let parsed: serde_json::Value = serde_json::from_slice(&data).map_err(|e| {
                        Error::BackendExecution(
                            self.platform().into(),
                            format!("JSON parse failed: {}", e),
                        )
                    })?;
                    return Ok(ChannelOutput {
                        platform: self.platform().into(),
                        action: action.into(),
                        backend: backend.name().into(),
                        data: parsed,
                        duration_ms: start.elapsed().as_millis() as u64,
                    });
                }
                Err(e) => {
                    tracing::warn!("Backend {} failed: {}", backend.name(), e);
                    return Err(e);
                }
            }
        }

        Err(Error::BackendUnavailable(
            self.platform().into(),
            "No backends available".into(),
        ))
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

/// Fetch URL content with optional proxy
async fn fetch_url(url: &str, config: &Config) -> ChannelResult<String> {
    let mut client_builder = reqwest::Client::builder().timeout(std::time::Duration::from_secs(30));

    if let Some(proxy_url) = &config.proxy {
        let proxy = reqwest::Proxy::all(proxy_url)
            .map_err(|e| Error::Config(format!("Invalid proxy: {}", e)))?;
        client_builder = client_builder.proxy(proxy);
    }

    let client = client_builder
        .build()
        .map_err(|e| Error::Network(e.to_string()))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| Error::Network(e.to_string()))?;

    if !response.status().is_success() {
        return Err(Error::BackendExecution(
            "rss".into(),
            format!(
                "HTTP {}: {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            ),
        ));
    }

    response
        .text()
        .await
        .map_err(|e| Error::Network(e.to_string()))
}

/// Parse RSS 2.0 or Atom XML into a normalized JSON structure
fn parse_feed(xml: &str) -> Result<serde_json::Value, String> {
    // Try RSS 2.0 first
    if let Ok(channel) = rss::Channel::read_from(xml.as_bytes()) {
        return Ok(serde_json::json!({
            "format": "rss",
            "feed": {
                "title": channel.title(),
                "link": channel.link(),
                "description": channel.description(),
            },
            "items": channel.items().iter().map(|item| {
                serde_json::json!({
                    "title": item.title().unwrap_or_default(),
                    "url": item.link().unwrap_or_default(),
                    "description": item.description().unwrap_or_default(),
                    "published_at": item.pub_date().unwrap_or_default(),
                })
            }).collect::<Vec<_>>(),
        }));
    }

    // Try Atom
    if let Ok(feed) = atom_syndication::Feed::read_from(xml.as_bytes()) {
        return Ok(serde_json::json!({
            "format": "atom",
            "feed": {
                "title": feed.title().value,
                "link": feed.links().first().map(|l| l.href()).unwrap_or_default(),
                "description": feed.subtitle().map(|t| t.value.as_str()).unwrap_or_default(),
            },
            "items": feed.entries().iter().map(|entry| {
                serde_json::json!({
                    "title": entry.title().value,
                    "url": entry.links().first().map(|l| l.href()).unwrap_or_default(),
                    "description": entry.summary().map(|t| t.value.as_str()).unwrap_or_default(),
                    "published_at": entry.updated().to_rfc3339(),
                })
            }).collect::<Vec<_>>(),
        }));
    }

    Err("Unrecognized feed format (neither RSS 2.0 nor Atom)".into())
}

#[cfg(test)]
mod tests {
    use agent_reach_core::{Channel, Config};

    use super::RssChannel;

    #[tokio::test]
    async fn parses_rss_feed_from_inline_xml() {
        let channel = RssChannel::new();
        let config = Config::default();
        let xml = r#"
            <rss version="2.0">
              <channel>
                <title>Örnek Akış</title>
                <link>https://example.com/</link>
                <description>Deneme RSS</description>
                <item>
                  <title>İlk Yazı</title>
                  <link>https://example.com/ilk</link>
                  <description>İlk özet</description>
                  <pubDate>Mon, 10 Aug 2026 10:00:00 GMT</pubDate>
                </item>
              </channel>
            </rss>
        "#;

        let output = channel
            .execute("parse", &[xml.to_string()], &config)
            .await
            .expect("RSS parse should succeed");

        assert_eq!(output.platform, "rss");
        assert_eq!(output.backend, "rss-parser");
        assert_eq!(output.data["feed"]["title"], "Örnek Akış");
        assert_eq!(output.data["items"][0]["title"], "İlk Yazı");
        assert_eq!(output.data["items"][0]["url"], "https://example.com/ilk");
    }

    #[tokio::test]
    async fn parses_atom_feed_from_inline_xml() {
        let channel = RssChannel::new();
        let config = Config::default();
        let xml = r#"
            <feed xmlns="http://www.w3.org/2005/Atom">
              <title>Örnek Atom</title>
              <link href="https://example.com/"/>
              <subtitle>Deneme Atom</subtitle>
              <updated>2026-08-10T10:00:00Z</updated>
              <entry>
                <title>Atom Yazı</title>
                <link href="https://example.com/atom"/>
                <summary>Atom özet</summary>
                <updated>2026-08-10T10:00:00Z</updated>
              </entry>
            </feed>
        "#;

        let output = channel
            .execute("parse", &[xml.to_string()], &config)
            .await
            .expect("Atom parse should succeed");

        assert_eq!(output.data["format"], "atom");
        assert_eq!(output.data["feed"]["title"], "Örnek Atom");
        assert_eq!(output.data["items"][0]["title"], "Atom Yazı");
        assert_eq!(output.data["items"][0]["url"], "https://example.com/atom");
    }

    #[tokio::test]
    async fn rejects_invalid_xml() {
        let channel = RssChannel::new();
        let config = Config::default();

        let result = channel
            .execute("parse", &["<not-a-feed>".to_string()], &config)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn rejects_unsupported_action() {
        let channel = RssChannel::new();
        let config = Config::default();

        let result = channel
            .execute(
                "transcribe",
                &["https://example.com/feed.xml".to_string()],
                &config,
            )
            .await;

        assert!(result.is_err());
    }
}
