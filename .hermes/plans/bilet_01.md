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
3. **Doğrudan Web Erişimi (yeniden yazıldı, 18 Ağu 2026 — Mihenk):**

   Bu madde ilk hâlinde *"arama motorları olmadan doğrudan gezinti"* diyordu ve
   raporun hafifletmesi bunu *"uygun HTTP header/user-agent rotasyonları"* ile
   çözmeyi öneriyordu. Bu, projenin yasak listesinin dördüncü maddesidir
   (*"tarayıcı imzası ya da başlık forgery'si"*), yani madde kendi kısıtıyla
   çelişiyordu. Ölçüm de bunu doğruladı: DuckDuckGo'nun HTML ucu yük altında
   `202` + sonuç taşımayan ara sayfa döndürüyor, üç paced denemede de.

   **Ayrım şu:**

   - **Yapılır — bilinen adresten okumak.** Elimizde zaten bir URL varsa
     (`web_read`, `crates.io` meta verisi, bir depo sayfası, bir RSS akışı)
     doğrudan gidip okumak meşrudur ve bugün çalışıyor. Kavram bağlarından
     *aday adres üretip* onları okumak da bu kapsamdadır.
   - **Yapılmaz — arama motorunun yerine geçmek için bot denetimini aşmak.**
     Başlık/UA rotasyonu, parmak izi sahteciliği, CAPTCHA çözme, vekil
     döndürme. Duvara çarpıldığında duvar gösterilir, aşılmaz.

   Pratik sonuç: kavram yönelimli gezinti **keşif** için değil, **doğrulama ve
   zenginleştirme** için kullanılır. Adayı arama motoru bulur; bağlantı grafiği
   onu derinleştirir.
