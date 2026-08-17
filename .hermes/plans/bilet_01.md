# Bilet 01: Entegrasyon Katmanı ve Motor Stratejisi

## Durum: KAPATILDI (Karar Alındı)

## Alınan Kararlar:
1. **Konumlandırma:** `crates/agent-reach-graph` adında bağımsız bir Rust crate'i olarak kurgulanacak. Projedeki tüm bileşenler (kanallar, CLI, MCP sunucusu) bu kütüphaneyi kullanabilecek.
2. **Çift Kanallı / Gölge Çalışma (Dual-Track / Shadow Execution):**
   - Aramalar başlangıçta iki sekmeli yapılacak:
     - 1. Sekme: Mevcut hızlı/statik arama (Hardcode gevşetme kuralları)
     - 2. Sekme: Semantik Zihin Haritası Motoru (Gölge Modu — Arka planda gizli)
   - Başlangıçta semantik çıktıların olgunlaşmama/saçmalama ihtimaline karşı zihin haritası yanıtları gizli kalacak, arka planda kendisini eğitmeye devam edecek.
   - Zihin haritasının başarısı statik kuralları geçtiğinde hardcode kademeli olarak devre dışı bırakılacak.
3. **Doğrudan Web Araması Altyapısı (Search-Engine Free Traversal):**
   - Arada DuckDuckGo/Exa/Google gibi arama motorları olmadan, kavram bağlantıları üzerinden web sayfalarında doğrudan gezinti ve içerik ayıklama yapabilecek esnek bir arayüz/mimarinin temelleri atılacak.
