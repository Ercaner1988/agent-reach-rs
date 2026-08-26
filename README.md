**🌍 [Türkçe](README.md) | [English](README.en.md) | [العربية](README.ar.md) | [日本語](README.ja.md)**

---

# 👁️ Agent Reach (Rust)

**Yapay zekâ ajanlarınız için %100 Rust-özgün internet okuma ve semantik arama katmanı**

> **Not:** Bu proje, [Agent Reach](https://github.com/Panniantong/agent-reach) Python sürümünün tam Rust yeniden yazımıdır (üst akıma katkı değil, bağımsız müstakil yeni bir uygulama). Amaç: sıfır Python bağımlılığı, tek ikili, saf Rust derlemesi, yüksek hız.

---

## 🎯 Varış Noktası ve Tamamlananlar

### Tamamlanan Çekirdek Bileşenler
- [x] **Çalışma alanı iskeleti** — 5 sandık (`core`, `channels`, `graph`, `mcp`, `cli`)
- [x] **Ağ kanalı** — Jina Reader (`r.jina.ai`) bütünleşmesi
- [x] **RSS kanalı** — RSS 2.0 ve Atom besleme okuma/çözümleme (`fetch` + `parse`)
- [x] **Twitter** — `twitter-cli` (kimlik doğrulamalı), Nitter (anonim)
- [x] **YouTube** — `yt-dlp` (tam çıkarım), `rustube` (üstveri)
- [x] **GitHub** — `gh` CLI (3-stage gevşetme merdiveni), GitHub REST API
- [x] **Reddit** — PRAW (Python), Reddit API (OAuth2)
- [x] **Çin Sosyal Medyası ve Finans** — Bilibili, Xiaohongshu, V2EX, Xueqiu, Xiaoyuzhou
- [x] **Profesyonel ve Arama** — LinkedIn, Exa Search, DuckDuckGo (HTML arama)
- [x] **Semantik Zihin Haritası** — `agent-reach-graph` (Pure-Rust 5-boyutlu epistemik vektör motoru)
- [x] **Komut satırı arayüzü** — `install`, `configure`, `doctor`, `skill`, `transcribe`, `execute`
- [x] **MCP sunucusu** — stdio JSON-RPC arayüzü, 14 kanal erişimi
- [x] **Çok platformlu derleme** — Windows, Linux, macOS (`cargo-dist`)
- [x] **Sürekli bütünleşme hattı** — GitHub Actions CI/CD ve otomatik test kapıları (`harness/` (Rust))

Ayrıntılı harita: [`docs/HARITA.md`](docs/HARITA.md) (Yolbulucu yapısı)

---

## 🏗️ Mimari

```
agent-reach-rs/
├── crates/
│   ├── agent-reach-core/      # Yapılandırma, Arka-uç/Kanal nitelikleri, Kaset Önbelleği
│   ├── agent-reach-channels/  # 14 platform okuyucu (web, youtube, twitter, github, ...)
│   ├── agent-reach-graph/     # Pure-Rust Semantik Çizge & Epistemik Vektör Motoru
│   ├── agent-reach-mcp/       # MCP stdio JSON-RPC sunucusu
│   └── agent-reach-cli/       # Clap CLI (kurulum, doktor, yetenek, çalıştırma)
├── harness/                   # Otomatik denetim kapıları ve kaset önbellek alanı
└── Cargo.toml                 # Çalışma alanı kökü
```

### Arka-uç Stratejisi

Her kanal birden çok arka-uç tanımlar (ilk seçim + yedek):
- **Twitter:** `twitter-cli` $\rightarrow$ yedek: `Nitter`
- **Reddit:** `Reddit API` $\rightarrow$ yedek: `PRAW`
- **YouTube:** `rustube` (üstveri) + `yt-dlp` alt-süreç (tam çıkarım)
- **GitHub:** `gh` CLI (tırnaksız bağımsız terim ayrıştırması) $\rightarrow$ yedek: `GitHub REST API`

---

## 🚀 Kurulum

### Kaynaktan Derleme

```bash
git clone https://github.com/Ercaner1988/agent-reach-rs.git
cd agent-reach-rs
cargo build --release
./target/release/agent-reach --help
```

### Kararlı Sürüm Kurulumu

```bash
cargo install agent-reach-cli
agent-reach install --env=auto
agent-reach doctor  # Sağlık ve bağımlılık denetimi
```

---

## 📖 Kullanım ve Aracı Çalıştırma

### Ağ Kanalı — Tek Sayfa Okuma

```bash
agent-reach execute --task-file - <<EOF
[
  {
    "id": "task-1",
    "channel": "web",
    "action": "read",
    "args": ["https://example.com"]
  }
]
EOF
```

### Toplu Görev Çalıştırma (`tasks.json`)

```bash
cat > tasks.json <<EOF
[
  {
    "id": "read-rust-docs",
    "channel": "web",
    "action": "read",
    "args": ["https://doc.rust-lang.org"]
  },
  {
    "id": "search-github",
    "channel": "github",
    "action": "search",
    "args": ["http client library"]
  }
]
EOF

agent-reach execute --task-file tasks.json --output execution_log.json --verbose
```

---

## 🛡️ Otomatik Test Kapıları

Projeye yapılan her ekleme 6 ücretsiz test kapısından (`harness/` (Rust)) geçer:

```bash
cargo run --manifest-path harness/Cargo.toml -- gates
```

- **Kapı 1 (Derleme):** `cargo build --workspace`
- **Kapı 2 (Clippy):** `cargo clippy --workspace --all-targets -- -D warnings`
- **Kapı 3 (Birim Testleri):** `cargo test --workspace`
- **Kapı 4 (Biçimlendirme):** `cargo fmt --check`
- **Kapı 5 (Hile Grep'i):** Cevap anahtarındaki kelimelerin koda sızmasını önleyen otomatik tarama
- **Kapı 6 (Eşik Bekçisi):** Hakem dosyalarının git referansı kontrolü

---

## 👥 Katkıda Bulunanlar

Ayrıntılı liste için [`CONTRIBUTORS.md`](CONTRIBUTORS.md) dosyasına bakabilirsiniz.
- **Ercan ER** ([@Ercaner1988](https://github.com/Ercaner1988)) — Proje Sahibi ve Baş Mimar
- **Kassam** (Hermes Agent / Nous Research) — Yapay Zekâ Meslektaş ve Geliştirici Ortak
- **Mihenk** (Claude Opus 5 / Anthropic) — Hakem ve Mimari İncelemeci
- **Devin AI** — Otomatik Geliştirici Katkıcısı
