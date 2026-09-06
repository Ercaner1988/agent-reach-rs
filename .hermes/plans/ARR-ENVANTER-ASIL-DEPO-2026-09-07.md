# ARR Envanteri — ASIL DEPO (Desktop/Github/agent-reach-rs), 7 Eylül 2026

Bu dosya, README güncellemesinin ikinci turunda kullanılacak ÖLÇÜLMÜİŞ gerçekleri tutar.
Önceki tur yanlış depoda (C:\Users\buzbe\agent-reach-rs) çalışıldı.
ASIL DEPO: C:\Users\buzbe\Desktop\Github\agent-reach-rs

## 1 · Depo durumu
- HEAD: dbe3b0c "Add pre-push gate hook and usage docs" = origin/main (0 ahead / 0 behind)
- Toplam commit: 77
- Lisans: MIT (Copyright 2026 Ercan ER)
- Rust: edition 2021, rust-version 1.75
- Python kirlilik: 2797 dosyalık Python 3.14 runtime (168 MB) SİLİNDİ, depo şimdi temiz.
- Yanlış kopya (C:\Users\buzbe\agent-reach-rs, 68 commit, ilgisiz geçmiş): 
  → kullanıcı "kaldıralım bütün gereksiz kalabalığı" dedi, README yazımından sonra silinecek.

## 2 · Kanal sayısı: 17 (lib.rs'de 17 pub use)
| # | platform | eylemler | backend |
|---|---|---|---|
| 1 | web | read | jina-reader |
| 2 | rss | fetch, parse | rss-parser |
| 3 | twitter | search, timeline, thread | twitter-cli + ? |
| 4 | youtube | metadata, transcript, search | 2 backend |
| 5 | github | repo, issue, pr, search | gh-cli + github-api |
| 6 | reddit | subreddit, search, post | praw + reddit-api |
| 7 | bilibili | video, info, search | bbdown + ? |
| 8 | xiaohongshu | note, search | xhs-web |
| 9 | linkedin | profile, company, search | linkedin-api |
| 10 | v2ex | topic, node, hot, latest | v2ex-api |
| 11 | xueqiu | quote, stock, timeline, search | ? |
| 12 | xiaoyuzhou | podcast, episode | ? |
| 13 | exa | search | exa-api + exa-mcp |
| 14 | duckduckgo | search | ddg-html |
| 15 | turath | search, book, author, page | turath-api |
| 16 | **mevzuat** | mevzuat, metin, fihrist | mevzuat-web (mevzuat.gov.tr) |
| 17 | **uinjkt** | identify, journals, articles, record | uinjkt-oai (OAI-PMH 2.0, journal.uinjkt.ac.id) |

## 3 · Kanal erişimi (doctor/execute/MCP tutarsızlığı — dürüstçe yazılacak)
- **lib.rs (kaynak):** 17 kanal derlenip ihraç ediliyor.
- **cmd_execute:** 17 kanal match kolunda var → CLI `execute` ile HEPSİ erişilebilir.
- **cmd_doctor:** 15 kanal sağlık denetiminde → **mevzuat ve uinjkt eksik**.
- **MCP:** 12 kanal enum'da → mevzuat, uinjkt, web, rss, exa eksik
  (web/rss/exa ayrı araçlar → ama mevzuat/uinjkt MCP'den erişilemiyor).
  README'de: "CLI üzerinden 17 kanal; MCP üzerinden 14 (mevzuat ve uinjkt henüz
  agent_reach_execute enum'una eklenmedi, ve doctor'da da eksikler)."

## 4 · Testler (bu oturumda koşuldu)
- agent_reach_channels (lib): 39 passed
- agent_reach_core (lib): 16 passed
- search_gauntlet (integration): 3 passed, 1 ignored (ağ gerektiren)
- TOPLAM: 58 geçti, 0 başarısız, 1 yok sayıldı

## 5 · Kapılar (8 — cargo run harness/Cargo.toml -- gates)
- build: green
- clippy: green
- unit tests: green
- formatting: green
- answer-key grep: green (138 phrase)
- referee watch: green
- criteria wiring: green
- ci parity: green → **TOPLAM 8 KAPI YEŞİL**

## 6 · Katkıda bulunanlar (git shortlog -sne --all)
- 43 Ercan ER <ercan.er@medeniyet.edu.tr>
- 22 Ercan ER <ercan@example.com>
-  9 Ercan Er <ercan.er@medeniyet.edu.tr>
-  2 copilot-swe-agent[bot] <198982749+Copilot@users.noreply.github.com>
-  1 Devin AI <158243242+devin-ai-integration[bot]@users.noreply.github.com>
TOPLAM: 77 commit. Ercan ER birleşince: 74. Copilot BOT: 2 (PR #2'den). Devin AI: 1.

## 7 · Yeni ADR
- docs/adr/0004-a-200-is-not-a-payload-verdict.md
  → require_payload() doğrulaması: HTTP 200 ama içerik boş/login duvarı → hata.

## 8 · Yeni özellikler (34d542d sonrası 9 commit)
- mevzuat kanalı (mevzuat.gov.tr, açık Türk mevzuat portalı)
- uinjkt kanalı (UIN Jakarta, OAI-PMH 2.0 İslami akademik dergiler)
- require_payload() — backend.rs'ye yükleme doğrulaması eklendi
- pre-push gate hook — push öncesi 8 kapıyı koşar, kırmızıysa durdurur
- CI parity gate — CI'daki cargo kontrollerinin harness'ta da olduğunu denetler
- post-commit / post-push hook'ları (pasli tetikleri)
- ZAI GLM 5.3 contributor tablosuna eklendi

## 9 · Altın küme
- 24 vaka, hepsi github hedefli, turath vakası yok (Bilet A açık)
- kabul.json: min_recall_ratio=0.9, max_zero_results=0

## 10 · Yol haritası güncellemesi (YOL-HARITASI-KAYNAKLAR.md)
Yanlış depodaki .md dosyasına yazılmıştı; asıl depoya taşınacak.
5 yeni kaynak: UYAP Emsal, Yargıtay, Danıştay, Pew, Lexpera.

## 11 · MCP araçları (5)
web_read(url) · rss_fetch(url) · rss_parse(xml) · exa_search(query, num_results)
agent_reach_execute(channel[12], action, args[])

## 12 · dist-workspace.toml
installers = [] → hazır ikili YOK, kaynaktan derleme tek yol.
Hedefler: aarch64-apple-darwin, aarch64-unknown-linux-gnu, x86_64-apple-darwin,
x86_64-unknown-linux-gnu, x86_64-pc-windows-msvc.
