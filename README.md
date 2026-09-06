**🌍 [Türkçe](README.md) | [English](README.en.md) | [العربية](README.ar.md) | [日本語](README.ja.md) | [中文](README.zh.md) | [Русский](README.ru.md) | [Español](README.es.md)**

# Agent Reach RS (`agent-reach-rs`)

> **Yapay zekâ ajanlarına internetin tamamını görecek gözler veren saf Rust okuma motoru**

`agent-reach-rs`, yapay zekâ ajanlarının (Hermes, Claude, Codex, OpenCode) web sayfaları, toplumsal ağlar, mevzuat ve akademik veri tabanları ile yerel ses/görüntü dosyaları üzerinden güvenilir biçimde veri okumasını sağlayan modüler bir Rust bütünüdür.

Belgenin dayandığı ölçüm: `git rev-parse --short HEAD` → `dbe3b0c`, `cargo test --workspace` → **58 geçti, 0 başarısız, 1 yok sayıldı**, `harness/gates` → **8 kapı yeşil** (7 Eylül 2026).

---

## 🎯 1. Varış Noktası ve Tamamlanan Özellikler

- **17 kanal okuyucu** — her kanal bir veya daha çok arka uç dener; biri düşerse sıradaki denenir (CLI → API → kazıyıcı merdiveni).
- **Harici FFmpeg bağımsızlığı (`MediaInspector`):** `symphonia` 0.5 ile MP3, WAV, AAC, FLAC, OGG, MKV gibi biçimler harici `ffmpeg` ikilisi olmadan çözümlenir. `format_name` alanı sabit `symphonia-native` döner.
- **Kanal anahtarı (`disabled_channels`):** İstenmeyen kaynak yapılandırmadan ya da ortam değişkeninden kapatılır. Kapalı kanal **bozuk kanaldan ayrı** bir hata verir — biri karardır, öteki kovalanacak arıza.
- **Yükleme doğrulaması (`require_payload`, ADR 0004):** HTTP 200 dönen ama gerçek içerik taşımayan (giriş duvarı, çevrimdışı bildirimi, 404 sayfası) yanıtlar tespit edilir ve sessizce başarılı sayılmaz.
- **Push öncesi altın kapı (`.githooks/pre-push`):** Push öncesinde 8 kalite kapısını yerelde koşar; biçim veya derleme ihlali varsa push'u durdurur.
- **CI eşitlik kapısı (`gate_ci_parity`):** CI iş akışındaki tüm `cargo` denetimlerinin yerel test kapılarında da bulunduğunu doğrular.
- **Kaset (`cassette`) düzeni:** Yanıtlar kaydedilir ve tekrar oynatılır; başarısızlıklar da kaydedilir, böylece "ölçülemedi" yolu ağ olmadan sınanabilir. Varsayılan kapalı.
- **MCP sunucusu:** Model Context Protocol 2024-11-05, stdio JSON-RPC, beş araç.
- **Yerel çeviri yazı (`transcribe`):** Yerel ses/görüntü dosyası Whisper (`whisper-large-v3-turbo`) ile Groq veya OpenAI üzerinden metne çevrilir.
- **Hakem korumalı ölçüm:** Altın küme, kabul eşikleri ve kurcalama denetimi ayrı dosyalarda; ölçen taraf ölçüleni değiştiremez.

### Kanallar (17)

| Kanal | Eylemler | Not |
| :--- | :--- | :--- |
| `web` | `read` | Jina Reader üzerinden sayfa metni |
| `rss` | `fetch`, `parse` | RSS 2.0 + Atom; Substack yayınları da bu kanaldan |
| `twitter` | `search`, `timeline`, `thread` | İki arka uç |
| `youtube` | `metadata`, `transcript`, `search` | İki arka uç |
| `github` | `repo`, `issue`, `pr`, `search` | `gh` CLI + REST API |
| `reddit` | `subreddit`, `search`, `post` | PRAW + REST API |
| `bilibili` | `video`, `info`, `search` | İki arka uç |
| `xiaohongshu` | `note`, `search` | Web arka ucu |
| `linkedin` | `profile`, `company`, `search` | Kimlik gerektirir |
| `v2ex` | `topic`, `node`, `hot`, `latest` | Açık API |
| `xueqiu` | `quote`, `stock`, `timeline`, `search` | Borsa verisi |
| `xiaoyuzhou` | `podcast`, `episode` | Sesli yayın |
| `exa` | `search` | Anlamsal arama, anahtar gerektirir |
| `duckduckgo` | `search` | HTML çıkarıcı, anahtarsız |
| `turath` | `search`, `book`, `author`, `page` | Klasik İslam eserleri; **cilt ve sayfa numarasıyla** döner |
| `mevzuat` | `mevzuat`, `metin`, `fihrist` | Türk mevzuatı (`mevzuat.gov.tr`), kanun no ile doğrudan sorgu |
| `uinjkt` | `identify`, `journals`, `articles`, `record` | UIN Jakarta İslami akademik dergileri (OAI-PMH 2.0, açık erişim) |

### Sırada bekleyen kaynaklar

`docs/YOL-HARITASI-KAYNAKLAR.md` belgesinde uçları ölçülmüş hâlde duruyor:
**UYAP Emsal**, **Yargıtay Karar Arama** (ikisi de anahtarsız JSON, `aramalist` + `getDokuman`), **Danıştay** (uç ayakta, arama gövdesi çözülecek), **Pew Research** (`robots.txt` ajanlara açıkça izin veriyor, `ai-input=yes`), **Lexpera** (abonelik duvarı + 10 saniyelik bekleme zorunluluğu → insan kapısı modeli), ayrıca Wikipedia, Stanford Felsefe Ansiklopedisi, TDV İslam Ansiklopedisi, OpenAlex, Crossref ve YÖK Tez.

---

## 🏗️ 2. Mimari ve Modüller

```text
agent-reach-rs/
├── Cargo.toml                    # Çalışma alanı (symphonia, tokio, reqwest, clap)
├── .githooks/                    # Paslı git kancaları (pre-push, post-commit, post-push)
├── crates/
│   ├── agent-reach-core/         # Channel, Backend, Config, Cassette, require_payload,
│   │                             #   MediaInspector, doctor, error
│   ├── agent-reach-channels/     # 17 kanal gerçeklemesi (mevzuat, uinjkt, turath dahil)
│   ├── agent-reach-mcp/          # MCP JSON-RPC sunucusu (stdio)
│   └── agent-reach-cli/          # Komut satırı istemcisi (ikili: agent-reach)
├── harness/                      # Hakem kapıları — ayrı derlenir (workspace dışı)
│   ├── kabul.json                # Kabul eşikleri (altın kümeden AYRI dosya)
│   └── biletler/                 # Bilet A · B · C ve sonuçları
└── docs/
    ├── adr/                      # 0001 · 0002 · 0003 · 0004 mimari kararları
    └── YOL-HARITASI-KAYNAKLAR.md # Kaynak kuşakları ve ölçülmüş uçlar
```

**Neden `harness` çalışma alanının dışında:** kapıları koşan araç, ölçtüğü ağaçla birlikte derlenmezse, kodda çıkan bir kırılma hakemi de düşürmez.

### Çekirdek türler

| Tür | Görevi |
| :--- | :--- |
| `Channel` | Bir kaynağın adı, eylemleri ve yürütücüsü |
| `Backend` | Kanalın altındaki tekil yol; `BackendStatus` ile sağlık bildirir |
| `Config` | `~/.agent-reach/config.yaml` okuma/yazma, `channel_enabled` denetimi |
| `Cassette` | Yanıt kaydı ve tekrarı (`AGENT_REACH_CASSETTE`) |
| `MediaInspector` | Saf Rust medya çözümleyici (`symphonia`) |
| `HealthCheck` | `doctor` çıktısının veri türü |

---

## 🚀 3. Kurulum ve Yapılandırma

### Ön gereksinimler

- **Rust 1.75+** (`cargo` ve `rustc`).
- **Harici ikili gerekmez:** FFmpeg, Python veya Node.js zorunluluğu yoktur. Yalnız bazı kanalların isteğe bağlı arka uçları (`gh` CLI, `praw`, `BBDown`) kuruluysa kullanılır; kurulu değilse kanal öbür arka uca düşer.

### Derleme

```bash
git clone https://github.com/Ercaner1988/agent-reach-rs.git
cd agent-reach-rs
cargo build --release
```

Üretilen ikili: `target/release/agent-reach` (Windows'ta `agent-reach.exe`).

> **Not:** Bu depo hazır indirilebilir ikili yayımlamıyor. `dist-workspace.toml` içinde `installers = []` tanımlı; yayın varlıkları yalnız arşiv olarak üretilir. Kaynaktan derleme tek yoldur.

### İlk yapılandırma

```bash
agent-reach install                      # ~/.agent-reach dizinini kurar
agent-reach configure github_token <tk>  # Anahtar tanımlama
agent-reach configure --json             # Mevcut ayarları görme
agent-reach doctor                       # Kanal sağlık denetimi
agent-reach doctor --json                # Makine okunur sağlık raporu
```

`install` komutu harici araçları **kurmaz**; yalnız yapılandırma dizinini hazırlar.

### Kanal kapatma

`~/.agent-reach/config.yaml`:

```yaml
disabled_channels:
  - reddit
  - linkedin
```

Ya da dosyaya dokunmadan, tek koşu için:

```bash
AGENT_REACH_DISABLED_CHANNELS=reddit,linkedin agent-reach execute --task-file gorevler.json
```

Ad karşılaştırması büyük/küçük harf ve boşluk duyarsızdır; önek eşleşmesi yoktur (`red` yazmak `reddit`'i kapatmaz). Hem CLI hem MCP yolunda geçerlidir.

### Tanınan yapılandırma anahtarları

`twitter_auth_token` · `twitter_ct0` · `groq_api_key` · `openai_api_key` · `github_token` · `reddit_client_id` · `reddit_client_secret` · `reddit_user_agent` · `linkedin_username` · `linkedin_password` · `exa_api_key` · `proxy` · `disabled_channels`

---

## 📖 4. Kullanım ve Örnekler

### A. Komut satırı

`agent-reach --help` çıktısındaki alt komutlar:

```text
install     Yapılandırma dizinini kurar (harici araçları kurmaz)
configure   Ayar tanımlar ya da tarayıcıdan çıkarır
doctor      Kanal erişilebilirliğini denetler
skill       Ajan yeteneği kaydını yönetir
execute     JSON dosyasından görev yürütür
transcribe  Yerel ses/görüntü dosyasını metne çevirir
```

### B. Görev yürütme

Kanallar görev dosyası üzerinden çalıştırılır (CLI üzerinden 17 kanalın tamamı desteklenir):

```json
[
  {
    "id": "gorev-1",
    "channel": "turath",
    "action": "search",
    "args": ["الاجتهاد"]
  },
  {
    "id": "gorev-2",
    "channel": "mevzuat",
    "action": "mevzuat",
    "args": ["5237"]
  },
  {
    "id": "gorev-3",
    "channel": "uinjkt",
    "action": "journals",
    "args": []
  }
]
```

```bash
agent-reach execute --task-file gorevler.json --output kutuk.json
agent-reach execute --task-file gorevler.json --continue-on-error --verbose
```

Çıktı (`kutuk.json`) şeması:

```json
{
  "total_duration_ms": 1240,
  "success": true,
  "results": [
    {
      "task_id": "gorev-1",
      "success": true,
      "channel": "turath",
      "backend": "turath-api",
      "duration_ms": 812,
      "output": "…",
      "error": null
    }
  ]
}
```

### C. Yerel çeviri yazı

```bash
agent-reach transcribe kayit.mp3
agent-reach transcribe ders.mkv --provider groq --output ders.txt
```

Yalnız yerel dosya kabul eder. `groq_api_key` ya da `openai_api_key` tanımlı değilse komut açıkça uyarır ve `configure` yolunu gösterir.

### D. Saf Rust medya çözümleme (`MediaInspector`)

```rust
use agent_reach_core::MediaInspector;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let meta = MediaInspector::inspect_file("ornek_ses.mp3")?;

    println!("Biçim: {}", meta.format_name);          // symphonia-native
    println!("Çözücü: {}", meta.codec_name);
    println!("Örnekleme: {} Hz", meta.sample_rate);
    println!("Kanal sayısı: {}", meta.channels);
    println!("Süre: {:.2} saniye", meta.duration_seconds);

    Ok(())
}
```

### E. MCP sunucusu

`agent-reach-mcp` ikilisi stdio üzerinden JSON-RPC konuşur (protokol `2024-11-05`). Beş araç sunar:

| Araç | Değiştirgeler |
| :--- | :--- |
| `web_read` | `url` |
| `rss_fetch` | `url` |
| `rss_parse` | `xml` |
| `exa_search` | `query`, `num_results` (1–10) |
| `agent_reach_execute` | `channel`, `action`, `args[]` |

`agent_reach_execute` şu on iki kanalı kabul eder: `bilibili`, `duckduckgo`, `github`, `linkedin`, `reddit`, `turath`, `twitter`, `v2ex`, `xiaohongshu`, `xiaoyuzhou`, `xueqiu`, `youtube`. `web`, `rss` ve `exa` kendi araçlarıyla erişilir — böylece on dört kanal MCP üzerinden ulaşılabilir.

> **Dürüstlük notu:** CLI `execute` üzerinden 17 kanalın tamamı çalışırken; `mevzuat` ve `uinjkt` kanalları henüz MCP `agent_reach_execute` enum'una ve `doctor` komutuna eklenmemiştir; sıradaki güncellemede eklenecektir.

Hermes / Claude Desktop yapılandırması:

```json
{
  "mcpServers": {
    "agent-reach": {
      "command": "/kurulum/yolu/agent-reach-mcp"
    }
  }
}
```

---

## 🛡️ 5. Kalite Kapıları ve Testler

### Ölçülen test durumu

```bash
cargo test --workspace
```

7 Eylül 2026'da bu depoda koşuldu, gerçek çıktı:

| Küme | Sonuç |
| :--- | :--- |
| `agent_reach_channels` (kitaplık) | 39 geçti · 0 başarısız |
| `agent_reach_core` (kitaplık) | 16 geçti · 0 başarısız |
| `search_gauntlet` (bütünleşme) | 3 geçti · 0 başarısız · 1 yok sayıldı |
| **Toplam** | **58 geçti · 0 başarısız · 1 yok sayıldı** |

Yok sayılan tek sınama canlı ağ ister; `--ignored` bayrağıyla elle koşulur.

### Hakem düzeni ve 8 altın kapı

`harness` ayrı bir crate'tir ve `cargo run --manifest-path harness/Cargo.toml -- gates` ile 8 kapıyı koşar:

1. **build** — Workspace derleme doğrulaması.
2. **clippy** — Sıfır uyarı zorunluluğu (`-D warnings`).
3. **unit tests** — Tüm birim sınamalarının geçmesi.
4. **formatting** — `cargo fmt` biçim uyumu.
5. **answer-key grep** — Altın küme yanıtlarının kaynak koda kopyalanmadığının denetimi (138 ifade taranır).
6. **referee watch** — Hakem dosyalarının kurcalanmadığının denetimi.
7. **criteria wiring** — Kabul ölçütlerinin gerçekten bağlı olduğunun doğrulanması.
8. **ci parity** — CI iş akışındaki denetimlerin yerel kapılarda da bulunduğunun güvencesi.

Eşikler `harness/kabul.json` içinde ve altın kümeden **ayrı** durur (`min_recall_ratio: 0.9`, `max_zero_results: 0`). Altın küme `crates/agent-reach-channels/tests/golden_search.json` içinde **24 vaka** tutar.

**Açıkça kayıtlı eksik:** yirmi dört vakanın tamamı `github` hedeflidir. Bilet A'nın şartı — "yeni bir kanal, altın kümede en az bir vaka kazanmadan çalışıyor sayılmaz" — `turath`, `mevzuat` ve `uinjkt` için **henüz karşılanmamıştır**. Kanallar canlıda ölçüldü, ancak altın kümeye henüz eklenmedi.

### Ölçüm ahlakı

- `docs/adr/0003-a-refusal-is-not-a-verdict.md` gereği: taşıma katmanının reddi (HTTP 429, 202) bir **yetenek eksiği sayılmaz**, `Ölçülemedi` olarak işaretlenir.
- `docs/adr/0004-a-200-is-not-a-payload-verdict.md` gereği: HTTP 200 dönüp veri taşımayan sahte başarılar (`require_payload`) hata sayılır.

---

## 👥 6. Katkıda Bulunanlar

Aşağıdaki sayılar `git shortlog -sne --all` çıktısındandır (7 Eylül 2026, toplam 77 kayıt).

| İsim / Kimlik | Rol ve Katkılar | Ölçüm |
| :--- | :--- | :--- |
| **Ercan ER** ([@Ercaner1988](https://github.com/Ercaner1988)) | Proje sahibi ve baş mimar; Rust mimarisi, 17 kanal tasarımı, kaynak kuşakları, ölçüm ahlakı (ADR 0001–0004) | **74 kayıt** (43 + 22 + 9, üç imza birleştirildi) |
| **copilot-swe-agent[bot]** ([@copilot-swe-agent[bot]](https://github.com/apps/copilot-swe-agent)) | PR #2: CI `cargo fmt` ve lint-and-format uyumluluk düzeltmeleri | **2 kayıt** |
| **Devin AI** ([@devin-ai-integration[bot]](https://github.com/apps/devin-ai-integration)) | PR #1: çapraz dizge Python yol tespiti, LinkedIn/Reddit enjeksiyon önlemleri, CLI yönlendirmeleri | **1 kayıt** |
| **Kassam** (Hermes ajanı) | Ajan meslektaş; `MediaInspector`, `turath` kanalı, kanal anahtarı, yedi dilli belge kümesi | Ercan ER imzasıyla işlendi |
| **Mihenk** (Claude Opus 5) | Mimari incelemeci ve hakem; ADR 0001–0004, hakem kilidi, Bilet A · B · C, yükleme doğrulaması | Ercan ER imzasıyla işlendi |
| **GitHub Copilot** (Düzenleyici Asistanı) | Düzenleyici içi kod tamamlama; yinelenen Rust kalıpları, `match` dalları, test iskeletleri | Ayrı git imzası yok |
| **ZAI GLM 5.3** (Zhipu AI) | Arama merdiveni ve kanal optimizasyon önerileri (commit `139b713`) | Ercan ER imzasıyla işlendi |

**Dürüstlük notu:** Kassam, Mihenk, ZAI GLM 5.3 ve GitHub Copilot düzenleyici asistanı ayrı git imzasıyla işlem yapmaz; ürettikleri değişiklikler Ercan ER'in imzasıyla kaydedilir. Doğrudan GitHub botu olarak PR açan araçlar (`copilot-swe-agent[bot]`: 2, `Devin AI`: 1) kendi imzalarıyla yer alır.

---

## 📄 7. Lisans

Bu proje **MIT Lisansı** altındadır — `LICENSE` dosyasına bakınız.
Telif: 2026 Ercan ER.

Belgeler: `CONTEXT.md` (kavram sözlüğü), `docs/adr/` (0001–0004 mimari kararlar), `docs/YOL-HARITASI-KAYNAKLAR.md` (kaynak kuşakları), `docs/channels/mevzuat.md`, `docs/channels/KANAL-EKLEME.md`, `docs/integration/mcp.md`, `docs/channels/rss.md`.
