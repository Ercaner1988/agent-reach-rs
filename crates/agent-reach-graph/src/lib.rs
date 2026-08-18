//! Single-axis weighted knowledge graph crate (`agent-reach-graph`)
//!
//! Epistemic single-axis weight graph powered by Turso 0.7.2 (pure Rust SQLite).
//! Mode: Shadow query expansion and feedback learning.

use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

pub struct Graph {
    conn: Arc<Mutex<turso::Connection>>,
}

impl Graph {
    /// Create or open a database at `db_path`, or memory database if `None`.
    pub async fn new(db_path: Option<&str>) -> Result<Self> {
        let db_location = match db_path {
            Some(p) => p.to_string(),
            None => {
                if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "agent-reach") {
                    let dir = proj_dirs.data_dir();
                    std::fs::create_dir_all(dir)?;
                    dir.join("graph.db").to_string_lossy().to_string()
                } else {
                    ":memory:".to_string()
                }
            }
        };

        let db = turso::Builder::new_local(&db_location).build().await?;
        let conn = db.connect()?;

        // Schema creation: dugumler & bagintilar
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
                kaynak TEXT NOT NULL,
                hedef TEXT NOT NULL,
                agirlik REAL NOT NULL DEFAULT 1.0,
                PRIMARY KEY (kaynak, hedef)
            );",
            (),
        )
        .await?;

        let graph = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        graph.seed_default_bridges().await?;

        Ok(graph)
    }

    /// Open default database handle.
    pub async fn open_default() -> Result<Self> {
        Self::new(None).await
    }

    /// Open in-memory database for testing.
    pub async fn in_memory() -> Result<Self> {
        Self::new(Some(":memory:")).await
    }

    /// Seed default concept bridge associations if table is empty.
    async fn seed_default_bridges(&self) -> Result<()> {
        let conn = self.conn.lock().await;

        let mut rows = conn.query("SELECT COUNT(*) FROM bagintilar;", ()).await?;
        let count: i64 = if let Some(row) = rows.next().await? {
            row.get(0)?
        } else {
            0
        };

        if count == 0 {
            debug!("Seeding initial concept bridge graph in Turso DB");

            let seeds = [
                ("gorsel", "tui"),
                ("görsel", "tui"),
                ("terminal", "tui"),
                ("arayuz", "tui"),
                ("arayüz", "tui"),
                ("kutuphanesi", "library"),
                ("kütüphanesi", "library"),
                ("iletisim", "http"),
                ("iletişim", "http"),
                ("guvenilir", "http"),
                ("güvenilir", "http"),
                ("hızlı", "http"),
                ("hizli", "http"),
                ("ağ", "http"),
                ("ag", "http"),
                ("atom", "rss"),
                ("news", "rss"),
                ("updates", "rss"),
                ("parse", "rss"),
                ("feed", "rss"),
            ];

            for (src, dst) in seeds {
                conn.execute(
                    "INSERT OR IGNORE INTO dugumler (terim) VALUES (?);",
                    turso::params![src],
                )
                .await?;
                conn.execute(
                    "INSERT OR IGNORE INTO dugumler (terim) VALUES (?);",
                    turso::params![dst],
                )
                .await?;
                conn.execute(
                    "INSERT OR REPLACE INTO bagintilar (kaynak, hedef, agirlik) VALUES (?, ?, 1.0);",
                    turso::params![src, dst],
                )
                .await?;
            }
        }

        Ok(())
    }

    /// Perform shadow expansion on input query tokens using graph edge weights.
    pub async fn expand_shadow(&self, query: &str) -> Vec<String> {
        let conn = self.conn.lock().await;
        let tokens: Vec<String> = query
            .split_whitespace()
            .map(|w| {
                w.trim_matches(|c: char| !c.is_alphanumeric() && c != '+' && c != '#')
                    .to_lowercase()
            })
            .filter(|w| !w.is_empty())
            .collect();

        let mut term_weights: HashMap<String, f64> = HashMap::new();

        for token in &tokens {
            if let Ok(mut rows) = conn
                .query(
                    "SELECT hedef, agirlik FROM bagintilar WHERE kaynak = ? ORDER BY agirlik DESC LIMIT 5;",
                    turso::params![token.as_str()],
                )
                .await
            {
                while let Ok(Some(row)) = rows.next().await {
                    if let (Ok(hedef), Ok(weight)) = (row.get::<String>(0), row.get::<f64>(1)) {
                        *term_weights.entry(hedef).or_insert(0.0) += weight;
                    }
                }
            }
        }

        if term_weights.is_empty() {
            return Vec::new();
        }

        let mut sorted_terms: Vec<(String, f64)> = term_weights.into_iter().collect();
        sorted_terms.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let top_terms: Vec<String> = sorted_terms.into_iter().map(|(term, _)| term).collect();

        let mut expansions = Vec::new();

        // Single best concept term
        if let Some(first) = top_terms.first() {
            expansions.push(first.clone());
        }

        // Combination with top terms
        if top_terms.len() >= 2 {
            expansions.push(top_terms[..2].join(" "));
        }

        expansions
    }

    /// Record positive feedback signal for used bridge edges.
    pub async fn record_success(&self, kaynak: &str, hedef: &str) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO bagintilar (kaynak, hedef, agirlik) VALUES (?, ?, 1.1)
             ON CONFLICT(kaynak, hedef) DO UPDATE SET agirlik = agirlik + 0.1;",
            turso::params![kaynak, hedef],
        )
        .await?;
        Ok(())
    }

    /// Decay unused edges over time.
    pub async fn decay_unused(&self, active_edges: &[(String, String)]) -> Result<()> {
        let conn = self.conn.lock().await;
        if active_edges.is_empty() {
            conn.execute("UPDATE bagintilar SET agirlik = agirlik * 0.95;", ())
                .await?;
        } else {
            let active_set: HashSet<(String, String)> = active_edges.iter().cloned().collect();
            let mut rows = conn
                .query("SELECT kaynak, hedef, agirlik FROM bagintilar;", ())
                .await?;

            let mut to_update = Vec::new();
            while let Some(row) = rows.next().await? {
                let k: String = row.get(0)?;
                let h: String = row.get(1)?;
                let w: f64 = row.get(2)?;
                if !active_set.contains(&(k.clone(), h.clone())) {
                    to_update.push((k, h, w * 0.95));
                }
            }

            for (k, h, new_w) in to_update {
                conn.execute(
                    "UPDATE bagintilar SET agirlik = ? WHERE kaynak = ? AND hedef = ?;",
                    turso::params![new_w, k, h],
                )
                .await?;
            }
        }
        Ok(())
    }

    /// Query current weight of an edge.
    pub async fn get_weight(&self, kaynak: &str, hedef: &str) -> Option<f64> {
        let conn = self.conn.lock().await;
        if let Ok(mut rows) = conn
            .query(
                "SELECT agirlik FROM bagintilar WHERE kaynak = ? AND hedef = ?;",
                turso::params![kaynak, hedef],
            )
            .await
        {
            if let Ok(Some(row)) = rows.next().await {
                return row.get(0).ok();
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_graph_initialization_and_shadow_expansion() {
        let graph = Graph::in_memory().await.unwrap();

        let shadow = graph.expand_shadow("gorsel terminal arayuz").await;
        assert!(!shadow.is_empty());
        assert_eq!(shadow[0], "tui");
    }

    #[tokio::test]
    async fn test_weight_update_and_decay() {
        let graph = Graph::in_memory().await.unwrap();

        let initial_weight = graph.get_weight("gorsel", "tui").await.unwrap();
        assert_eq!(initial_weight, 1.0);

        graph.record_success("gorsel", "tui").await.unwrap();
        let updated_weight = graph.get_weight("gorsel", "tui").await.unwrap();
        assert!(updated_weight > 1.0);

        graph.decay_unused(&[]).await.unwrap();
        let decayed_weight = graph.get_weight("gorsel", "tui").await.unwrap();
        assert!(decayed_weight < updated_weight);
    }
}
