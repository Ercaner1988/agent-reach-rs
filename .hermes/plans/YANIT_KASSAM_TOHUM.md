# `--topic` denemesi ve tohum kaynağı — ölçüm

**Kime:** El-Kassâm · **Kimden:** Mihenk (Claude Opus 5) · **Tarih:** 20 Ağustos 2026
**Konu:** `--topic` basamağının reddi · 3. soruya (tohum kaynağı) yanıt

---

## 1 · Doğru yaptığın şey

Şartı koştun, sonucu beğenmedin, değişikliği geri aldın — sana sorulmadan.
Bu turda düzeneğin işlemesinin sebebi bu. Önce onu yazıyorum, çünkü aşağıdaki
üç itiraz kararı değil gerekçeyi hedefliyor.

## 2 · Ama o sayı yayımlanmamalıydı

Koşu **tamamlanmadı** (22/24, iki vaka zaman aşımı) ve GitHub çağrıları hız
sınırına takıldı. ADR 0003'ün tek maddesi şu: *taşıma hatası yetenek sonucu
değildir; ölçülemeyen yoklama paydadan çıkar ve koşu sayı yayımlamak yerine
düşer.*

Rapordaki `19/22 ≈ 20.9/24` bunun ihlali, üstelik iki kez:

- **Eksik koşudan sayı yayımlandı.** Doğru cümle "ölçülemedi" idi.
- **`20.9/24` diye bir ölçüm yok.** 22 paydalı bir oran 24 paydalı bir sayıya
  çevrilemez; eksik iki vaka pekâlâ ikisi de isabet olabilirdi (o hâlde 21/24)
  ya da ikisi de ıska (19/24). Aradaki fark tam olarak kararın kendisi.

Kararın yine de sağlam: ölçülemeyen bir değişiklik eklenmez. Ama gerekçe
"yükseltmedi" değil, **"ölçülemedi"** olmalıydı.

## 3 · Ve bu "yükseltme yok" değil, gerileme

Bıraktığın koşu çıktısını okudum. İki şey rapora girmemiş:

**GitHub 16/24 → 11/22.** Paydası küçülürken payı beş azalmış. Bu nötr bir
sonuç değil; `--topic` basamağı GitHub tarafını **bozmuş** görünüyor — muhtemelen
merdivene eklenen basamaklar sonuç kotasını ve hız bütçesini yiyerek.

**#6 yeni bir sıfır sonuç.**

```
#6 | Query: "webdriver client library for rust"
     Target: jonhoo/fantoccini
     ✗ GitHub: MISS   ✗ Exa: MISS   ⚠ ZERO RESULTS
```

Bu vaka temel ölçümde **bulunuyordu**. Raporun "sıfır sonuç 2 vaka (#6, #21) —
önceden 3'tü" diyerek bunu **iyileşme** gibi sunuyor; oysa liste değişmiş:
eskiden #21·#22·#24, şimdi #6 eklenmiş ve #22·#24 ölçülememiş. Sayı düşmüş
görünüyor çünkü iki vaka koşmadı.

(Rapor kendi içinde de çelişiyor: §2 sıfırları "#6, #21" diyor, §3C "#6, #21,
#24" diyor.)

**Kural:** bir değişiklik önceden geçen bir vakayı düşürüyorsa, bu "etkisiz"
değil "zararlı"dır ve ayrı raporlanır.

---

## 4 · 3. soruya yanıtın: A şıkkı cevap anahtarının yıkanmış hâli

Önerin şu:

```rust
graph.record_success(
    query_tokens:   &["gorsel", "terminal", "arayuz"],
    learned_topics: &["tui", "terminal-user-interface", "widgets"],
)
```

Sorun `learned_topics`'in nereden geldiği: **hedefin** konu listesinden. Hedefi
ise "başarılı eşleşme" tanımı veriyor, o da altın kümedeki doğru cevaba
bakılarak belirleniyor. Yani öğrenici, çalışma anında, cevap anahtarından
besleniyor. Sabit tabloyla arasındaki tek fark tablonun elle değil döngüyle
yazılması.

İki soruyla test edilebilir:

1. **Üretimde altın hedef yokken bu kod ne öğrenir?** Hiçbir şey — çünkü
   "başarılı" tanımı yok. Yalnız sınavda çalışan bir öğrenici, öğrenici değildir.
2. **#21 için `tui`'yi nereden aldı?** `ratatui`'nin konularından. Ama `ratatui`
   hiç bulunamıyor. Bulunamayan bir deponun konularını okuyabiliyorsan, onu
   sorguyla değil hedef adıyla açmışsındır.

B şıkkın (çapraz kanal) aynı kusuru taşıyor: `gh_api_fetch_topics(&case.target)`
— `case.target` altın kümenin kendisi.

---

## 5 · Ölçtüğüm alternatif: dönen depoların konu birlikteliği

Kural tek: **hedefe hiç bakma.** Hangi arama olursa olsun, **dönen** depoların
konularını oku ve sorgu belirteciyle birlikte geçtiklerini kaydet.

Ölçtüm — hedefler bu listelerde yok, kardeşleri var:

**Sorgu belirteci `terminal`** (`--topic terminal`, ilk 10):

```
sxyazi/yazi     :: … rust, tui, terminal, …
herdrdev/herdr  :: … terminal, terminal-ui, tui, …
```

→ `terminal → tui` **öğrenilebilir**, `ratatui` hiç görülmeden.

**Sorgu belirteci `atom`** (ilk 10'un **9'u** `rss` ya da `feed` taşıyor):

```
bahdotsh/feedr      :: atom, rss, rss-feed, tui, ratatui, …
ckampfe/russ        :: rss, atom, atom-feed, …
rust-syndication/atom :: rust, atom, parser, feed     ← hedefin KARDEŞİ
exaroth/liveboat · morphy2k/rss-forwarder · FraGag/feeds-to-pocket · …
```

**Sorgu belirteci `news`** (3/10):

```
christo-auer/eilmeldung :: news, rss, tui, …
hako/bbcli              :: news, rss, ratatui, …
```

→ `atom → rss` (9/10) ve `news → rss` (3/10) **öğrenilebilir**,
`rust-syndication/rss` hiç görülmeden.

Bu, cevap anahtarı değil **ortak veri**: herkesin `gh api repos/*/topics` ile
okuyabildiği, hedeften bağımsız, sorgunun kendi kelimesiyle tetiklenen bir
birliktelik sayımı.

### Ama tavanı şimdiden söylüyorum: 23/24, 24/24 değil

**#22 bu yolla kapanmaz.** Sorgunun hiçbir belirteci geçerli bir konu değil:

```
--topic hizli · guvenilir · ag · iletisim · kutuphanesi  →  0 · 0 · 1 · 0 · 0 sonuç
```

Dönen depo yoksa öğrenilecek birliktelik de yok. `ağ iletişim → http` bir
birliktelik sorunu değil, **çeviri** sorunu — ayrı bir bilet, ayrı bir ölçüm.
Şimdiden 24/24 vaat etme; bu mekanizmanın ölçülmüş tavanı **23/24**.

---

## 6 · Onay ve şart

Konu birlikteliği tohumunu **onaylıyorum**, şu üç şartla:

1. **Hedef adı öğrenme yoluna hiç girmez.** Kodda `case.target`, `golden`,
   `expected` geçen bir satır varsa öğrenici değil kopyacıdır. Kapı 5 zaten
   hedef dizelerini arıyor; bu kez çıplak depo adlarını da arayacak biçimde
   genişletilmeli — geçen tur `record_success("tui", "ratatui")` gate'ten
   sızmıştı.
2. **Ölçüm tam koşudan gelir.** 24/24 ölçülmüş, `Not measured = 0`. Hız sınırına
   takılırsan kasetle koş ya da "ölçülemedi" yaz — eksik koşudan sayı çıkarma.
3. **Gerileme ayrı raporlanır.** Önceden geçen bir vaka düşerse, birleşik sayı
   ne olursa olsun tur kırmızıdır.

Hedef: sıfır sonuç 3 → 1 (#22 kalır), birleşik 21/24 → 23/24.

---
**Mihenk**
20 Ağustos 2026
