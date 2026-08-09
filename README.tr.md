# 👁️ Agent Reach (Rust)

**Yapay zeka ajanlarınız için %100 Rust-native internet okuma katmanı**

> **Not:** Bu proje, [Agent Reach](https://github.com/Panniantong/agent-reach) Python sürümünün tam Rust yeniden yazımıdır (upstream'e katkı değil, yeni bir implementation). Hedef: sıfır Python bağımlılığı, tek binary, hızlı kurulum.

---

## 🎯 Varış Noktası (Yol Haritası)

- [x] **Workspace skeleton** — 4 crate (core/channels/mcp/cli) + trait'ler
- [x] **Web kanalı** — Jina Reader (r.jina.ai) entegrasyonu
- [ ] **14 kanal** — Twitter, Reddit, YouTube, RSS, GitHub, Bilibili, Xiaohongshu, LinkedIn, V2EX, Xueqiu, Xiaoyuzhou, Exa Search
- [ ] **CLI** — `agent-reach install/configure/doctor/skill/transcribe`
- [ ] **MCP sunucusu** — stdio JSON-RPC (Exa tool)
- [ ] **SkillOptOrchestrator** — Hermes native skill execution entegrasyonu

Detaylı harita: [`docs/HARITA.md`](docs/HARITA.md) (Yolbulucu/Wayfinder mimarisi)

---

## 🏗️ Mimari

```
agent-reach-rs/
├── crates/
│   ├── agent-reach-core/      # Config, Backend/Channel trait'leri, Doctor
│   ├── agent-reach-channels/  # 14 platform okuyucu (web, youtube, twitter, ...)
│   ├── agent-reach-mcp/       # MCP stdio sunucusu (exa_search tool)
│   └── agent-reach-cli/       # Clap CLI (install, configure, doctor, skill)
└── Cargo.toml                 # Workspace root
```

**Backend Stratejisi:** Her kanal birden fazla backend tanımlar (ilk seçim + yedek):
- **Twitter:** `twitter-cli` → yedek: `OpenCLI`
- **Reddit:** `OpenCLI` → yedek: `rdt-cli`
- **YouTube:** `rustube` (metadata) + `yt-dlp` subprocess (tam extraction)

---

## 🚀 Kurulum (Planlanan)

```bash
cargo install agent-reach-cli
agent-reach install --env=auto
agent-reach doctor  # Sağlık kontrolü
```

**Şu anda:** Geliştirme aşamasında. `cargo check --all` çalışıyor, ilk kanal (Web) impl edildi.

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
- **[Yolbulucu Harita](docs/HARITA.md)** — Çok oturumlu orkestrasyon, bilet sistemi
- **[Bağımlılık Tablosu](docs/dependencies.md)** — Python paketi → Rust crate eşleştirmeleri

---

## 🌍 Çok Dilli Dokümantasyon

- **Türkçe (ana):** Bu dosya
- **Arapça:** [`README.ar.md`](README.ar.md)
- **İngilizce:** [`README.md`](README.md)

---

## 🤝 Katkı

Proje aktif geliştirme altında. PR ve issue'lar hoş karşılanır.

**Önemli:** Upstream Python Agent Reach'e katkı için [orijinal repo](https://github.com/Panniantong/agent-reach)'ya gidin. Bu repo yalnız Rust native implementasyondur.

---

## 📜 Lisans

MIT License — bkz. [LICENSE](LICENSE)

---

## 🔗 İlgili Projeler

- [Agent Reach (Python)](https://github.com/Panniantong/agent-reach) — Orijinal implementasyon
- [ZOPAY](https://github.com/Ercaner1988/zotero-zero-mcp) — Zotero MCP server (Rust)
- [Hermes Agent](https://github.com/NousResearch/hermes-agent) — AI ajan framework

---

**Durum:** 🟡 Geliştirme aşamasında (v0.1.0-pre)  
**Son güncelleme:** 2026-08-09  
**Yazar:** Ercan ER
