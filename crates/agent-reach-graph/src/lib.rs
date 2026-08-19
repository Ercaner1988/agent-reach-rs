use turso::Builder;
use std::sync::Arc;

pub struct Graph {
    db_path: String,
}

impl Graph {
    pub async fn open(path: &str) -> anyhow::Result<Self> {
        let db = Builder::new_local(path).build().await?;
        let conn = db.connect()?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS dugumler (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                terim TEXT UNIQUE NOT NULL
            );",
            (),
        ).await?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS bagintilar (
                kaynak INTEGER NOT NULL,
                hedef INTEGER NOT NULL,
                agirlik REAL NOT NULL DEFAULT 1.0,
                PRIMARY KEY (kaynak, hedef)
            );",
            (),
        ).await?;

        let graph = Graph {
            db_path: path.to_string(),
        };
        graph.seed_defaults().await?;
        Ok(graph)
    }

    pub async fn open_default() -> anyhow::Result<Self> {
        Self::open(":memory:").await
    }

    async fn get_conn(&self) -> anyhow::Result<turso::Connection> {
        let db = Builder::new_local(&self.db_path).build().await?;
        Ok(db.connect()?)
    }

    async fn seed_defaults(&self) -> anyhow::Result<()> {
        let conn = self.get_conn().await?;
        // Seed default concept bridges
        let default_bridges = [
            ("gorsel", "tui"),
            ("görsel", "tui"),
            ("terminal", "tui"),
            ("arayuz", "ui"),
            ("arayüz", "ui"),
            ("kutuphanesi", "library"),
            ("kütüphanesi", "library"),
            ("iletisim", "http"),
            ("iletişim", "http"),
            ("ag", "http"),
            ("ağ", "http"),
            ("guvenilir", "safe"),
            ("güvenilir", "safe"),
            ("hızlı", "fast"),
            ("hizli", "fast"),
            ("metin", "text"),
            ("arama", "search"),
            ("news", "rss"),
            ("updates", "rss"),
            ("atom", "rss"),
            ("feed", "rss"),
            ("syndication", "rss"),
            ("parse", "rss"),
        ];

        for (src, dst) in default_bridges {
            self.add_bridge_with_conn(&conn, src, dst, 1.0).await?;
        }

        Ok(())
    }

    async fn get_or_create_node(&self, conn: &turso::Connection, term: &str) -> anyhow::Result<i64> {
        let term_lower = term.to_lowercase();
        let mut rows = conn.query("SELECT id FROM dugumler WHERE terim = ?1", [term_lower.as_str()]).await?;
        if let Some(row) = rows.next().await? {
            let id: i64 = row.get(0)?;
            return Ok(id);
        }
        conn.execute("INSERT INTO dugumler (terim) VALUES (?1)", [term_lower.as_str()]).await?;
        let mut rows = conn.query("SELECT id FROM dugumler WHERE terim = ?1", [term_lower.as_str()]).await?;
        if let Some(row) = rows.next().await? {
            let id: i64 = row.get(0)?;
            return Ok(id);
        }
        anyhow::bail!("Failed to get node id for {}", term)
    }

    async fn add_bridge_with_conn(&self, conn: &turso::Connection, src: &str, dst: &str, weight: f64) -> anyhow::Result<()> {
        let src_id = self.get_or_create_node(conn, src).await?;
        let dst_id = self.get_or_create_node(conn, dst).await?;
        conn.execute(
            "INSERT INTO bagintilar (kaynak, hedef, agirlik) VALUES (?1, ?2, ?3)
             ON CONFLICT(kaynak, hedef) DO UPDATE SET agirlik = MAX(agirlik, excluded.agirlik)",
            turso::params![src_id, dst_id, weight],
        ).await?;
        Ok(())
    }

    /// Expand a query string using the graph (shadow mode expansion).
    /// Returns expanded terms mapped from source terms in the graph.
    pub async fn expand_terms(&self, terms: &[&str]) -> Vec<String> {
        let Ok(conn) = self.get_conn().await else {
            return Vec::new();
        };

        let mut expanded = Vec::new();
        for &term in terms {
            let term_lower = term.to_lowercase();
            let mut rows = match conn.query(
                "SELECT d2.terim, b.agirlik FROM dugumler d1 
                 JOIN bagintilar b ON d1.id = b.kaynak 
                 JOIN dugumler d2 ON b.hedef = d2.id 
                 WHERE d1.terim = ?1 AND b.agirlik > 0.1 
                 ORDER BY b.agirlik DESC",
                [term_lower.as_str()],
            ).await {
                Ok(r) => r,
                Err(_) => continue,
            };

            while let Ok(Some(row)) = rows.next().await {
                if let Ok(target_term) = row.get::<String>(0) {
                    if !expanded.contains(&target_term) && target_term != term_lower {
                        expanded.push(target_term);
                    }
                }
            }
        }
        expanded
    }

    /// Reward (reinforce) a connection when a search succeeds.
    pub async fn reinforce(&self, src: &str, dst: &str) -> anyhow::Result<()> {
        let conn = self.get_conn().await?;
        let src_id = self.get_or_create_node(&conn, src).await?;
        let dst_id = self.get_or_create_node(&conn, dst).await?;
        conn.execute(
            "INSERT INTO bagintilar (kaynak, hedef, agirlik) VALUES (?1, ?2, 1.1)
             ON CONFLICT(kaynak, hedef) DO UPDATE SET agirlik = agirlik * 1.1",
            turso::params![src_id, dst_id],
        ).await?;
        Ok(())
    }

    /// Decay unused connections (sönümlenme).
    pub async fn decay(&self, factor: f64) -> anyhow::Result<()> {
        let conn = self.get_conn().await?;
        conn.execute(
            "UPDATE bagintilar SET agirlik = agirlik * ?1",
            [factor],
        ).await?;
        Ok(())
    }
}
