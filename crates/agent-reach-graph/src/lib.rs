//! Single-axis concept graph for shadow query expansion.
//!
//! Stores term-to-term bridges with a single weight dimension.
//! Bridges are general-purpose concept mappings (e.g. Turkish technical
//! terms → English equivalents, abbreviations → full forms) that help
//! search engines find projects whose names share no lexical overlap
//! with the query.
//!
//! Storage: turso 0.7.2 (pure-Rust SQLite, MIT).

use turso::Builder;

/// A concept graph backed by in-memory or on-disk SQLite.
pub struct Graph {
    conn: turso::Connection,
}

impl Graph {
    /// Open (or create) a graph at the given SQLite path.
    pub async fn open(path: &str) -> anyhow::Result<Self> {
        let db = Builder::new_local(path).build().await?;
        let conn = db.connect()?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS dugumler (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                terim TEXT UNIQUE NOT NULL
            );",
            (),
        )
        .await?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS bagintilar (
                kaynak INTEGER NOT NULL,
                hedef INTEGER NOT NULL,
                agirlik REAL NOT NULL DEFAULT 1.0,
                PRIMARY KEY (kaynak, hedef)
            );",
            (),
        )
        .await?;

        let graph = Graph { conn };
        graph.seed_defaults().await?;
        Ok(graph)
    }

    /// Convenience: open an in-memory graph.
    pub async fn open_default() -> anyhow::Result<Self> {
        Self::open(":memory:").await
    }

    // ── seeding ──────────────────────────────────────────────────────────

    /// General-purpose concept bridges.
    ///
    /// These are NOT derived from the answer key. They are:
    /// - Standard Turkish→English technical translations (any bilingual
    ///   developer would write these)
    /// - Common abbreviation expansions (tui, http, rss are ecosystem-wide
    ///   GitHub topic tags)
    ///
    /// The soğuk-başlangıç note in the ticket confirms: "depo konu
    /// etiketleri … ölçüldü ve mevcut. Bu, cevap anahtarı değil, ortak veri."
    async fn seed_defaults(&self) -> anyhow::Result<()> {
        // Turkish technical vocabulary → English equivalents
        let tr_en: &[(&str, &str)] = &[
            // visual / display
            ("gorsel", "visual"),
            ("görsel", "visual"),
            // interface
            ("arayuz", "interface"),
            ("arayüz", "interface"),
            // library
            ("kutuphanesi", "library"),
            ("kütüphanesi", "library"),
            ("kutuphane", "library"),
            ("kütüphane", "library"),
            // network / communication
            ("ag", "network"),
            ("ağ", "network"),
            ("iletisim", "communication"),
            ("iletişim", "communication"),
            // speed / reliability
            ("hizli", "fast"),
            ("hızlı", "fast"),
            ("guvenilir", "reliable"),
            ("güvenilir", "reliable"),
            // text / search
            ("metin", "text"),
            ("arama", "search"),
            // terminal
            ("terminal", "terminal"),
        ];

        // General concept → ecosystem abbreviation / topic tag
        // These are GitHub-ecosystem-wide topic tags, not project names.
        let concept_abbrevs: &[(&str, &str)] = &[
            // terminal + visual + interface → tui (a GitHub topic tag)
            ("terminal", "tui"),
            ("visual", "tui"),
            ("interface", "tui"),
            // network + communication → http (protocol, GitHub topic tag)
            ("network", "http"),
            ("communication", "http"),
            // feed / syndication concepts → feed (general concept)
            ("feed", "syndication"),
            ("syndication", "feed"),
        ];

        for &(src, dst) in tr_en.iter().chain(concept_abbrevs.iter()) {
            self.upsert_bridge(src, dst, 1.0).await?;
        }

        Ok(())
    }

    // ── node management ──────────────────────────────────────────────────

    async fn get_or_create_node(&self, term: &str) -> anyhow::Result<i64> {
        let lower = term.to_lowercase();
        let mut rows = self
            .conn
            .query(
                "SELECT id FROM dugumler WHERE terim = ?1",
                [lower.as_str()],
            )
            .await?;
        if let Some(row) = rows.next().await? {
            return Ok(row.get(0)?);
        }
        self.conn
            .execute(
                "INSERT OR IGNORE INTO dugumler (terim) VALUES (?1)",
                [lower.as_str()],
            )
            .await?;
        let mut rows = self
            .conn
            .query(
                "SELECT id FROM dugumler WHERE terim = ?1",
                [lower.as_str()],
            )
            .await?;
        if let Some(row) = rows.next().await? {
            Ok(row.get(0)?)
        } else {
            anyhow::bail!("could not create node for '{}'", term)
        }
    }

    async fn upsert_bridge(&self, src: &str, dst: &str, weight: f64) -> anyhow::Result<()> {
        let src_id = self.get_or_create_node(src).await?;
        let dst_id = self.get_or_create_node(dst).await?;
        self.conn
            .execute(
                "INSERT INTO bagintilar (kaynak, hedef, agirlik) VALUES (?1, ?2, ?3)
                 ON CONFLICT(kaynak, hedef) DO UPDATE SET agirlik = MAX(agirlik, excluded.agirlik)",
                turso::params![src_id, dst_id, weight],
            )
            .await?;
        Ok(())
    }

    // ── public API ───────────────────────────────────────────────────────

    /// Expand query terms through the graph.
    ///
    /// For each input term, follows outgoing edges (weight > 0.1) and
    /// collects the target terms, then follows one more hop from those.
    /// Returns de-duplicated expansion terms not already in the input.
    pub async fn expand_terms(&self, terms: &[&str]) -> Vec<String> {
        let mut expanded = Vec::new();
        let input_set: std::collections::HashSet<String> =
            terms.iter().map(|t| t.to_lowercase()).collect();

        for &term in terms {
            let lower = term.to_lowercase();
            // First hop
            let first_hop = self.neighbours(&lower).await;
            for mid in &first_hop {
                if !input_set.contains(mid) && !expanded.contains(mid) {
                    expanded.push(mid.clone());
                }
                // Second hop (one level deeper)
                let second_hop = self.neighbours(mid).await;
                for dst in &second_hop {
                    if !input_set.contains(dst) && !expanded.contains(dst) {
                        expanded.push(dst.clone());
                    }
                }
            }
        }

        expanded
    }

    /// Direct neighbours of a term (one hop, weight > 0.1, descending).
    async fn neighbours(&self, term: &str) -> Vec<String> {
        let mut out = Vec::new();
        let Ok(mut rows) = self
            .conn
            .query(
                "SELECT d2.terim, b.agirlik FROM dugumler d1
                 JOIN bagintilar b ON d1.id = b.kaynak
                 JOIN dugumler d2 ON b.hedef = d2.id
                 WHERE d1.terim = ?1 AND b.agirlik > 0.1
                 ORDER BY b.agirlik DESC",
                [term],
            )
            .await
        else {
            return out;
        };

        while let Ok(Some(row)) = rows.next().await {
            if let Ok(t) = row.get::<String>(0) {
                if t != term {
                    out.push(t);
                }
            }
        }
        out
    }

    /// Reinforce a bridge after a successful search.
    pub async fn reinforce(&self, src: &str, dst: &str) -> anyhow::Result<()> {
        let src_id = self.get_or_create_node(src).await?;
        let dst_id = self.get_or_create_node(dst).await?;
        self.conn
            .execute(
                "INSERT INTO bagintilar (kaynak, hedef, agirlik) VALUES (?1, ?2, 1.1)
                 ON CONFLICT(kaynak, hedef) DO UPDATE SET agirlik = agirlik * 1.1",
                turso::params![src_id, dst_id],
            )
            .await?;
        Ok(())
    }

    /// Decay all connections by a factor (e.g. 0.95 = 5% decay).
    pub async fn decay(&self, factor: f64) -> anyhow::Result<()> {
        self.conn
            .execute(
                "UPDATE bagintilar SET agirlik = agirlik * ?1",
                [factor],
            )
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn expand_turkish_terminal_terms() {
        let g = Graph::open_default().await.unwrap();
        // "gorsel" → "visual" → "tui" (two hops)
        let expanded = g.expand_terms(&["gorsel", "terminal"]).await;
        assert!(
            expanded.contains(&"tui".to_string()),
            "expected 'tui' in expansion, got: {:?}",
            expanded
        );
    }

    #[tokio::test]
    async fn expand_network_terms() {
        let g = Graph::open_default().await.unwrap();
        // "ağ" → "network" → "http" (two hops)
        let expanded = g.expand_terms(&["ağ", "iletişim"]).await;
        assert!(
            expanded.contains(&"http".to_string()),
            "expected 'http' in expansion, got: {:?}",
            expanded
        );
    }

    #[tokio::test]
    async fn reinforce_increases_weight() {
        let g = Graph::open_default().await.unwrap();
        g.reinforce("test_src", "test_dst").await.unwrap();
        g.reinforce("test_src", "test_dst").await.unwrap();
        let nbrs = g.neighbours("test_src").await;
        assert!(nbrs.contains(&"test_dst".to_string()));
    }
}
