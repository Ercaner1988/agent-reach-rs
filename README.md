**🌍 [Türkçe](README.md) | [English](README.en.md) | [العربية](README.ar.md) | [日本語](README.ja.md) | [中文](README.zh.md) | [Русский](README.ru.md) | [Español](README.es.md)**

# Agent Reach RS (`agent-reach-rs`)

> **Yapay Zekâ Ajanları İçin Saf Rust Medya, Web ve Çok Kanallı Veri Okuma Motoru**

`agent-reach-rs`, yapay zekâ ajanlarının (Hermes, Claude, Codex, OpenCode) dış web siteleri, sosyal ağlar, akademik kaynaklar ve medya dosyaları üzerinden güvenilir, hızlı ve bağımsız veri okumasını sağlayan modüler bir Rust ekosistemidir.

---

## 🎯 1. Varış Noktası ve Tamamlanan Özellikler

- **Harici FFmpeg Binary Bağımsızlığı (`MediaInspector`):** Harici `ffmpeg.exe` ikilisine ihtiyaç duymadan, `symphonia` (v0.5) kütüphanesi ile MP3, WAV, AAC, FLAC, OGG ve MKV gibi medya formatlarını saf Rust ile doğrudan bellek ve disk üzerinden ayrıştırır.
- **14 Çoklu Kanal Okuyucu:**
  - **Sosyal & Web:** Twitter/X (Nitter / GraphQL), Reddit API, Bilibili, Xiaohongshu (XHS), V2EX, Xueqiu, LinkedIn, Xiaoyuzhou.
  - **Akademik & Kod:** Turath (İslam Hukuku ve Yazma Eser Veritabanı), GitHub REST API, RSS/Atom Yayınları.
  - **Arama Motorları:** Exa AI Semantik Arama, DuckDuckGo HTML Çıkarıcı, Jina Web Reader.
- **5D Epistemik Vektör Motoru (`agent-reach-graph`):** Turso SQLite (0.7.2) tabanlı ontolojik, estetik, epistemolojik, ahlaki ve dilsel boyut matrisi.
- **MCP Sunucu Entegrasyonu:** Model Context Protocol (MCP) standartlarına tam uyumlu JSON-RPC CLI ve sunucu bileşeni.

---

## 🏗️ 2. Mimari ve Modüller

```text
agent-reach-rs/
├── Cargo.toml                    # Workspace konfigürasyonu (symphonia, tokio, reqwest)
├── crates/
│   ├── agent-reach-core/        # Çekirdek veri türleri, MediaInspector, Hata yönetimi, Config
│   ├── agent-reach-channels/    # 14 kanal okuyucu gerçeklemesi (YouTube, Turath, RSS vb.)
│   ├── agent-reach-mcp/         # MCP JSON-RPC sunucu sürücüsü
│   └── agent-reach-cli/         # Komut satırı istemcisi (binary: agent-reach)
└── harness/                     # Otomatik test ve hakem doğrulama kapıları
```

---

## 🚀 3. Kurulum ve Yapılandırma

### Ön Gereksinimler
- **Rust Toolchain:** Rust 1.75+ (Cargo ve `rustc` kurulu olmalıdır).
- **Harici İkili Gereksinimi:** YOK (FFmpeg ikilisi, Python veya Node.js bağımlılığı bulunmamaktadır).

### Derleme
```bash
# Depoyu klonlayın
git clone https://github.com/Ercaner1988/agent-reach-rs.git
cd agent-reach-rs

# Workspace derlemesi yapın
cargo build --release
```

Derlenen binary `target/release/agent-reach.exe` yolunda oluşur.

---

## 📖 4. Kullanım ve Örnekler

### A. Saf Rust Medya Ayrıştırma (`MediaInspector` API)
```rust
use agent_reach_core::MediaInspector;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Harici ffmpeg.exe kullanmadan ses dosyasını analiz etme
    let meta = MediaInspector::inspect_file("ornek_ses.mp3")?;
    
    println!("Codec: {}", meta.codec_name);
    println!("Örnekleme Oranı: {} Hz", meta.sample_rate);
    println!("Kanal Sayısı: {}", meta.channels);
    println!("Süre: {:.2} saniye", meta.duration_seconds);
    
    Ok(())
}
```

### B. CLI Kullanımı
```bash
# Exa semantik arama çalıştırma
agent-reach --channel exa search "Max Weber hukuki aklileşme"

# Turath veritabanında eser okuma
agent-reach --channel turath read --book 124 --page 45

# RSS akışı okuma
agent-reach --channel rss fetch "https://news.ycombinator.com/rss"
```

---

## 🛡️ 5. Kalite Kapıları ve Testler

Proje, 6 sıkı doğrulama kapısı ve %100 geçme zorunluluğu ile korunmaktadır.

```bash
# Tüm workspace testlerini çalıştırma (41/41 yeşil kapı)
cargo test --workspace
```

- **`agent-reach-core`:** 10/10 test başarılı (Saf Rust medya ayrıştırma dahil).
- **`agent-reach-channels`:** 28/28 test başarılı.
- **`search_gauntlet`:** 3/3 hakem kapısı onaylı.

---

## 👥 6. Katkıda Bulunanlar

| İsim / Kimlik | Rol ve Katkılar | Metrikler |
| :--- | :--- | :--- |
| **Ercan Er** | Baş Mimar ve Proje Sahibi (Rust mimarisi, Nisa 135 ilkesi) | 38 commit, Ana Kod Tabanı |
| **Mihenk** | Kod İnceleme ve Hakem Denetçisi | Hakem Onayları & Gauntlet Denetimi |
| **El-Kassâm** | Ajan Geliştirici (MediaInspector, Saf Rust Entegrasyonu) | 12 commit, Medya & Test Entegrasyonu |
| **GitHub Copilot** | İkincil Kod Tamamlama Desteği | Yardımcı Geliştirme |
| **Hermes** | Ajan Orkestrasyonu ve Çalışma Ortamı | Ajan Yürütme Motoru |

---

## 📄 7. Lisans

Bu proje **MIT Lisansı** altında lisanslanmıştır. Detaylar için `LICENSE` dosyasına bakınız.
