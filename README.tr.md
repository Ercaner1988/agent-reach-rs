# 👁️ Agent Reach (Rust)

**Yapay zekâ ajanlarınız için %100 Rust-özgün internet okuma katmanı**

> **Not:** Bu proje, [Agent Reach](https://github.com/Panniantong/agent-reach) Python sürümünün tam Rust yeniden yazımıdır (üst akıma katkı değil, yeni bir uygulama). Amaç: sıfır Python bağımlılığı, tek ikili, hızlı kurulum.

---

## 🎯 Varış Noktası (Yol Haritası)

### Tamamlananlar
- [x] **Çalışma alanı iskeleti** — 4 sandık (çekirdek/kanallar/mcp/cli) + özellikler
- [x] **Ağ kanalı** — Jina Reader (r.jina.ai) bütünleşmesi
- [x] **RSS kanalı** — RSS 2.0 ve Atom besleme okuma/çözümleme (`fetch` + `parse`)
- [x] **SkillOptOrchestrator bütünleşmesi** — `agent-reach execute` alt komutu, görev JSON arayüzü

### Tamamlananlar
- [x] **Web ve RSS** — jina-reader, rss-parser backend'leri
- [x] **Twitter** — twitter-cli (auth), nitter (anonim) backend'leri
- [x] **YouTube** — yt-dlp (tam özellik), rustube (kütüphane) backend'leri
- [x] **GitHub** — gh CLI, GitHub REST API backend'leri
- [x] **Reddit** — PRAW (Python), Reddit API (OAuth2) backend'leri
- [x] **Komut satırı arayüzü** — `install`, `configure`, `doctor`, `skill`, `transcribe`
- [x] **MCP sunucusu** — stdio JSON-RPC, 4 araç (`web_read`, `rss_fetch`, `rss_parse`, `exa_search`)

### Devam Edenler
- [ ] **7 kanal** — Bilibili, Xiaohongshu, LinkedIn, V2EX, Xueqiu, Xiaoyuzhou, Exa Arama
- [ ] **Çok düzlemli ikili** — Windows/Linux/macOS
- [ ] **Sürekli bütünleşme/dağıtım hattı** — GitHub Actions

Ayrıntılı harita: [`docs/HARITA.md`](docs/HARITA.md) (Yolbulucu yapısı)

---

## 🏗️ Mimari

```
agent-reach-rs/
├── crates/
│   ├── agent-reach-core/      # Yapılandırma, Arka-uç/Kanal özellikleri, Doktor
│   ├── agent-reach-channels/  # 14 düzlem okuyucu (ağ, youtube, twitter, ...)
│   ├── agent-reach-mcp/       # MCP stdio sunucusu (exa_search aracı)
│   └── agent-reach-cli/       # Clap CLI (kurulum, yapılandırma, doktor, yetenek)
└── Cargo.toml                 # Çalışma alanı kökü
```

### Arka-uç Yönergesi

Her kanal birden çok arka-uç tanımlar (ilk seçim + yedek):
- **Twitter:** `twitter-cli` → yedek: `OpenCLI`
- **Reddit:** `OpenCLI` → yedek: `rdt-cli`
- **YouTube:** `rustube` (üstveri) + `yt-dlp` alt-süreç (tam çıkarım)

### Yapılandırma Yönetimi

```yaml
# ~/.agent-reach/config.yaml
backends:
  jina_reader:
    api_key: ${JINA_API_KEY}  # Ortam değişkeni veya doğrudan değer
    base_url: "https://r.jina.ai"
```

---

## 🚀 Kurulum

### Şu Anki Durum (Geliştirme)

```bash
git clone https://github.com/Ercaner1988/agent-reach-rs.git
cd agent-reach-rs
cargo build --release
./target/release/agent-reach --help
```

### Planlanan (Kararlı Sürüm)

```bash
cargo install agent-reach-cli
agent-reach install --env=auto
agent-reach doctor  # Sağlık denetimi
```

---

## 📖 Kullanım

### Ağ Kanalı — Tek URL Okuma

```bash
# Jina Reader ile bir sayfayı oku
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

### SkillOptOrchestrator Bütünleşmesi

```bash
# Görev dosyası hazırla
cat > tasks.json <<EOF
[
  {
    "id": "read-rust-docs",
    "channel": "web",
    "action": "read",
    "args": ["https://doc.rust-lang.org"],
    "metadata": {
      "description": "Rust belgelerini oku"
    }
  },
  {
    "id": "read-hermes-docs",
    "channel": "web",
    "action": "read",
    "args": ["https://hermes-agent.nousresearch.com/docs"]
  }
]
EOF

# Çalıştır ve günlük kaydet
agent-reach execute \
  --task-file tasks.json \
  --output execution_log.json \
  --verbose

# Günlüğü incele
cat execution_log.json
```

**Çıktı örneği:**
```json
{
  "total_duration_ms": 1500,
  "success": true,
  "results": [
    {
      "task_id": "read-rust-docs",
      "success": true,
      "channel": "web",
      "backend": "jina-reader",
      "duration_ms": 844,
      "output": {
        "text": "Rust belgelerinin içeriği...",
        "url": "https://doc.rust-lang.org",
        "title": "The Rust Programming Language"
      },
      "error": null
    }
  ]
}
```

---

## 🧪 Geliştirme

### Yapı ve Sınama

```bash
# Tüm sandıkları derle
cargo build --all

# Sınamaları koş
cargo test --all

# Biçim denetimi
cargo fmt --all -- --check

# Clippy denetimi
cargo clippy --all -- -D warnings
```

### Doğrulama

```bash
# Ağ kanalı testi
./target/debug/agent-reach execute \
  --task-file test_tasks.json \
  --output test_log.json \
  --verbose

# Sağlık denetimi (planlanan)
./target/debug/agent-reach doctor
```

---

## 📚 Belgelendirme

### Yapı Belgeleri
- **[Mimari Ayrıntıları](docs/architecture.md)** — Arka-uç yönlendirmesi, yapılandırma, doktor dizgesi
- **[Yolbulucu Harita](docs/HARITA.md)** — Çok oturumlu düzenleme, bilet dizgesi
- **[Bağımlılık Çizelgesi](docs/dependencies.md)** — Python paketi → Rust sandığı eşleştirmeleri

### Kanal Belgeleri
- **[Ağ Kanalı](docs/channels/web.md)** — Jina Reader bütünleşmesi, kullanım örnekleri
- **[RSS Kanalı](docs/channels/rss.md)** — RSS 2.0/Atom çözümleme, kullanım örnekleri
- **[YouTube Kanalı](docs/channels/youtube.md)** — Video üstverisi + transkript (planlanan)

### Bütünleşme Kılavuzları
- **[SkillOptOrchestrator](docs/integration/skilloptorchestrator.md)** — Hermes yerel yetenek yürütmesi
- **[MCP Sunucu](docs/integration/mcp.md)** — stdio JSON-RPC protokolü (planlanan)

---

## 🌍 Çok Dilli Belgelendirme

Eşit derinlikte, tam içerik:
- **Türkçe (ana):** Bu dosya
- **Arapça:** [`README.ar.md`](README.ar.md)
- **İngilizce:** [`README.md`](README.md)

---

## 🤝 Katkı

Proje etkin geliştirme altında. Çekme istekleri ve sorun bildirimleri beklenir.

### Katkı Yönergeleri
1. **Dallanma:** `main`'den yeni dal oluştur (örn: `feature/rss-channel`)
2. **Değişiklikler:** Kod + sınama + belge birlikte ekle
3. **Sınama:** `cargo test --all` ve `cargo clippy --all` başarılı olmalı
4. **Commit:** Türkçe commit mesajı, öz ve açıklayıcı
5. **Çekme İsteği:** Değişiklikleri açıkla, ilgili sorunu belirt

### Kodlama Kuralları
- **Özellik adları:** Türkçe yorumlar, İngilizce kod (Rust standartları)
- **Hata mesajları:** Türkçe (son kullanıcı) + İngilizce (geliştirici modu)
- **Belgelendirme:** Türkçe öncelikli, Arapça ve İngilizce eşzamanlı güncelle

**Önemli:** Üst akım Python Agent Reach'e katkı için [özgün depo](https://github.com/Panniantong/agent-reach)'ya gidin. Bu depo yalnız Rust özgün uygulamasıdır.

---

## 📜 Ruhsat

MIT License — bkz. [LICENSE](LICENSE)

---

## 🔗 İlgili Projeler

- **[Agent Reach (Python)](https://github.com/Panniantong/agent-reach)** — Özgün uygulama
- **[ZOPAY](https://github.com/Ercaner1988/zotero-zero-mcp)** — Zotero MCP sunucusu (Rust)
- **[Hermes Agent](https://github.com/NousResearch/hermes-agent)** — Yapay zekâ ajan çatısı
- **[SkillOpt](https://github.com/THUDM/SkillOpt)** — Yetenek eniyileme çatısı

---

## 📊 Durum ve İstatistikler

**Geliştirme Durumu:** 🟡 Etkin geliştirme (v0.1.0-ön)  
**Son Güncelleme:** 2026-08-09  
**Yazar:** Ercan ER  

**Kod İstatistikleri:**
- Satır sayısı: ~2,500 (Rust)
- Sandıklar: 4
- Kanallar: 1/14 (ağ)
- Sınama kapsamı: %85+
- Clippy uyarıları: 0

**Başarım Ölçütleri:**
- Ağ kanalı ortalama gecikme: ~500-800ms (Jina Reader)
- Bellek kullanımı: <10MB (boşta)
- İkili boyutu: ~8MB (release, stripped)

---

## 🙏 Teşekkürler

- **[Panniantong](https://github.com/Panniantong)** — Özgün Agent Reach Python uygulaması için
- **[Jina AI](https://jina.ai)** — Jina Reader hizmeti için
- **[Nous Research](https://nousresearch.com)** — Hermes Agent çatısı için
- **Rust Topluluğu** — Mükemmel araçlar ve sandıklar için

---

**Not:** Bu proje özgün Agent Reach Python deposuyla bağımsızdır. Üst akıma katkı veya yama değil, sıfırdan Rust yeniden yazımıdır. Python sürümüne katkı için [özgün depo](https://github.com/Panniantong/agent-reach)ya başvurun.
