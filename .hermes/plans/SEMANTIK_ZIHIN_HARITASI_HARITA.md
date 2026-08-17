# 🧭 YOLBULUCU (Wayfinder) — Semantik Zihin Haritası ve Değişkenlik Motoru Haritası

## Varış Noktası (Destination)
`agent-reach-rs` için bağımsız bir pure-Rust SQLite (`crates/agent-reach-graph`) tabanlı, **4 Boyutlu Epistemik Semantik Çizge ve Doğrudan İnternet Gezinti Katmanı** kurmak. Sert kodlu arama kurallarından kademeli olarak öğrenen, çok boyutlu epistemik vektör uzayında kavram bağıntılarını dinamik yöneten ve arama motorlarından bağımsız doğrudan internet kazıma yapabilen canlı bir zihin motoru oluşturmak.

## Özel Notlar & Tercihler (Notes)
- **Dil ve Terim Disiplini:** Öztürkçe terimler öncelikli (`uygulama`, `bütünleşme`, `özgün`, `bağıntı`, `kalıcılık katmanı`, `epistemik uzay`).
- **Çok Dil Desteği:** Türkçe (TR), İngilizce (ENG), Arapça (AR) için müstakil semantik uzaylar.
- **Çift Kanallı / Gölge Çalışma (Dual-Track / Shadow Execution):** Semantik kütüphane olgunlaşana kadar arka planda gizli çalışır, çıktılar olgunlaştıkça statik kuralları devreden çıkarır.
- **Ahlaki İlke (Nisâ 135):** Karar biletleri kanıta ve entelektüel tutarlılığa dayanır.

## Alınan Kararlar (Decisions So Far)
- [Bilet 01: Entegrasyon Katmanı ve Motor Stratejisi](.hermes/plans/bilet_01.md) — Bağımsız `crates/agent-reach-graph` crate'i; çift kanallı gölge çalışma; doğrudan web arama altyapısı hazırlığı.
- [Bilet 02: 4 Boyutlu Epistemik Semantik Çizge Şeması](.hermes/plans/bilet_02.md) — x-ekseni `[-1.0, +1.0]` (zıt/eş anlam) + 4 temel epistemik boyut (Ontolojik, Estetik, Epistemolojik, Ahlaki) + Dil bazlı ayrıştırma (TR/ENG/AR).
- [Bilet 03: Otomatik Öğrenme ve Gölge Sorgu Akışı](.hermes/plans/bilet_03.md) — Tam otomatik sorgu genişletme ve arka planda sessiz öğrenme döngüsü.

## Ön Cephe (Frontier — Tasarım & Kodlama Biletleri)
- [Bilet 04: Crate İskeleti ve Veritabanı Şeması Uygulaması](.hermes/plans/bilet_04.md) — `crates/agent-reach-graph` SQLite şeması, pure-Rust sürücü ve Rust struct yapılarının kodlanması.
- [Bilet 05: Çok Boyutlu Epistemik Uzay Hesaplama Motoru](.hermes/plans/bilet_05.md) — Düğümler arası $x$-ekseni ve 4-boyutlu uzaklık matrisi hesaplama mantığı.
- [Bilet 06: Çift Kanallı Arama ve Gölge Öğrenme Motoru Entegrasyonu](.hermes/plans/bilet_06.md) — `agent-reach-channels` içerisinde statik arama ile gölge semantik aramanın paralel koşulması.

## Henüz Belirlenmeyenler / Savaş Sisi (Not Yet Specified)
- Doğrudan arama motoru olmadan web sayfalarını gezme ve semantik olarak haritalama (Direct Web Crawler & Parser) algoritmik detayları.
- Zamanla değişen bağlam sönümlendirme (decay) matematiksel formülü.

## Kapsam Dışı (Out of Scope)
- Ağır C++ / dış API bağımlılıkları (Qdrant vb.).
- Statik ve genel arama motorlarına kalıcı olarak bağımlı kalmak.
