# 👁️ Agent Reach (Rust)

**AI ajanlarınız için %100 Rust-native internet okuma katmanı**

> **Not:** Bu proje, [Agent Reach](https://github.com/Panniantong/agent-reach) Python sürümünün tam Rust yeniden yazımıdır (upstream'e katkı değil, yeni bir implementation). Hedef: sıfır Python bağımlılığı, tek binary, hızlı kurulum.

---

## 🎯 Varış Noktası (Roadmap)

- [x] **Workspace skeleton** — 4 crate (core/channels/mcp/cli) + trait'ler
- [x] **Web channel** — Jina Reader (r.jina.ai) entegrasyonu
- [ ] **14 channel** — Twitter, Reddit, YouTube, RSS, GitHub, Bilibili, Xiaohongshu, LinkedIn, V2EX, Xueqiu, Xiaoyuzhou, Exa Search
- [ ] **CLI** — `agent-reach install/configure/doctor/skill/transcribe`
- [ ] **MCP server** — stdio JSON-RPC (Exa tool)
- [ ] **SkillOptOrchestrator** — Hermes native skill execution entegrasyonu

Detaylı harita: [`docs/HARITA.md`](docs/HARITA.md) (Yolbulucu/Wayfinder mimarisi)

---

## 🏗️ Mimari

```
agent-reach-rs/
├── crates/
│   ├── agent-reach-core/      # Config, Backend/Channel trait'leri, Doctor
│   ├── agent-reach-channels/  # 14 platform reader (web, youtube, twitter, ...)
│   ├── agent-reach-mcp/       # MCP stdio server (exa_search tool)
│   └── agent-reach-cli/       # Clap CLI (install, configure, doctor, skill)
└── Cargo.toml                 # Workspace root
```

**Backend Stratejisi:** Her channel birden fazla backend tanımlar (first-choice + fallback):
- **Twitter:** `twitter-cli` → fallback: `OpenCLI`
- **Reddit:** `OpenCLI` → fallback: `rdt-cli`
- **YouTube:** `rustube` (metadata) + `yt-dlp` subprocess (full extraction)

---

## 🚀 Kurulum (Planned)

```bash
cargo install agent-reach-cli
agent-reach install --env=auto
agent-reach doctor  # Health check
```

**Şu anda:** Geliştirme aşamasında. `cargo check --all` çalışıyor, ilk channel (Web) impl edildi.

---

## 🧪 Geliştirme

```bash
git clone https://github.com/Ercaner1988/agent-reach-rs.git
cd agent-reach-rs
cargo build --all
cargo test --all
cargo run --bin agent-reach-cli
```

---

## 📚 Dokümantasyon

- **[Mimari Detayları](docs/architecture.md)** — Backend routing, config, doctor sistemi
- **[Yolbulucu Harita](docs/HARITA.md)** — Multi-session orkestrasyon, bilet sistemi
- **[Bağımlılık Tablosu](docs/dependencies.md)** — Python paketi → Rust crate eşleştirmeleri

---

## 🌍 Çok Dilli Dokümantasyon

- **Türkçe (ana):** [`README.tr.md`](README.tr.md)
- **Arapça:** [`README.ar.md`](README.ar.md)
- **İngilizce:** Bu dosya

---

## 🤝 Katkı

Proje aktif geliştirim altında. PR ve issue'lar hoş karşılanır.

**Önemli:** Upstream Python Agent Reach'e katkı için [orijinal repo](https://github.com/Panniantong/agent-reach)'ya gidin. Bu repo yalnız Rust native implementasyondır.

---

## 📜 Lisans

MIT License — bkz. [LICENSE](LICENSE)

---

## 🔗 İlgili Projekte

- [Agent Reach (Python)](https://github.com/Panniantong/agent-reach) — Orijinal implementasyon
- [ZOPAY](https://github.com/Ercaner1988/zotero-zero-mcp) — Zotero MCP server (Rust)
- [Hermes Agent](https://github.com/NousResearch/hermes-agent) — AI ajan framework

---

**Durum:** 🟡 Geliştirme aşamasında (v0.1.0-pre)  
**Son güncelleme:** 2026-08-09  
**Yazar:** Ercan ER
