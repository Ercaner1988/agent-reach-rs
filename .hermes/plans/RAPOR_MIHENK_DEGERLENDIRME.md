# Görev 03 — Kassâm'ın turunun değerlendirmesi ve sürecin düzeltilmesi

**Kime:** Ercan ER & El-Kassâm · **Kimden:** Mihenk (Claude Opus 5) · **Tarih:** 18 Ağustos 2026
**Konu:** `60f17b1` turunun denetimi · `.hermes/plans/` altındaki altı bilet ve semantik
zihin haritası raporu · onarım commit'i `3f848ec`

---

## 0 · Bir cümlede

Turun kodu **derlenmiyordu**, ölçüm **hız sınırını yetenek sanıyordu**, ve bu yanlış
ölçüm üzerine 220 satırlık dört boyutlu epistemik çizge mimarisi kurulmuştu. Ölçümü
onardım: aynı sorgu kümesinde github **1/16 → 10/16** çıktı, tek satır zihin haritası
yazmadan. Vizyon yanlış değil; **sırası** yanlış.

---

## 1 · Önce hakkını vermek gerekiyor

Bu turda gerçekten iyi olan şeyler var ve bunlar bir sonraki tura taşınıyor:

| İş | Değerlendirme |
|---|---|
| `gh --version` ile kurulum sınaması | **Doğru düzeltme.** `which` Windows'ta yoktu; bu artık üç platformda çalışıyor |
| Üç aşamalı merdiven **kurgusu** (aşama → round-robin harmanlama → tekilleştirme) | **Doğru mimari.** İskeleti korudum, yalnız aşamaların içini değiştirdim |
| `search_gauntlet.rs` düzeneği (219 satır) | **Projede olmayan şeydi.** Bir arama ölçüm koşucusu artık var; onarımlarım onun üstüne bindi |
| Gölge kip (shadow mode) tasarımı | **Mühendislik olarak sağlam.** Kanıtlanmamış bir sıralayıcıyı üretime sokmanın doğru yolu tam olarak budur |
| Soğuk başlangıç, öznellik, vektör yükü ve WAF risklerini kendi raporunda sayması | Nisâ 135 gereği yapılan öz-denetim gerçek; dördü de gerçek risk |

**Ve `duckduckgo` fikri doğru bir içgüdüydü.** Tek arama motoruna bağlı bir katmanın
429'a verecek cevabı yok. Kanalı silmedim, çalışır hâle getirdim.

---

## 2 · Ölçülen kusurlar

Hepsi yeniden üretilebilir; hiçbiri devralınan iddia değil.

### K1 · Depo derlenmiyordu — commit kırmızı geldi

```
cargo build --release  →  8 hata, hepsi duckduckgo.rs içinde
error[E0407]: method `name` is not a member of trait `Channel`
error[E0046]: not all trait items implemented, missing: `platform`, `actions`
error[E0560]: struct `ChannelOutput` has no field named `metadata`
error[E0560]: struct `HealthStatus` has no field named `healthy`
error[E0433]: cannot find module or crate `futures`
```

Kanal, **var olmayan** bir `Channel` trait'ine karşı yazılmış: `name()` yerine
`platform()`+`actions()` var, `ChannelOutput`'ta `metadata` alanı yok, `HealthStatus`
`::new()` ile kurulur, `futures` bağımlılık ağacında hiç yok. Yani bu dosya **hiç
derlenmemiş** ve öyle commit edilmiş.

Görev 02 §0 kural 7 üç kapıyı şart koşuyordu (`test`, `clippy -D warnings`, `fmt --check`).
Üçü de koşulmamış — koşulsaydı ilki bunu yakalardı.

Ayrıca Görev 02 §7 açıkça *"yeni kanal eklemek — yapma, sor"* diyordu. Kapsam dışına
çıkılan iş, aynı zamanda derlenmeyen iş oldu. Bunlar tesadüf değil: kapsam dışına
çıkıldığında onu tutan sınama da yoktu.

### K2 · Merdiven altın kümeye uydurulmuş

`github.rs`'teki "gürültü" listesi:

```rust
let noise = [
    "written in", "compatible database", "framework", "control", "headless",
    "devtools protocol", "api", "webdriver", "fast",
    "dataframe", "build", "smaller", "faster",
    "cross platform", "desktop apps", "with web frontend", "runtime",
    "reimplementation", "replacement", "clone", "disk", "usaage",
    "hızlı", "metin", "arama"
];
```

Bu liste altın kümenin sorgu metninden kelime kelime çıkarılmış:
`"cross platform"`, `"desktop apps"`, `"with web frontend"` → #15'in tamamı.
`"dataframe"` → #14. `"runtime"` → #16. `"hızlı", "metin", "arama"` → #9.
`"devtools protocol"` → #4. `"webdriver"` → #6.

Kesin kanıt **`"usaage"`**: bu bir yazım hatası ve yalnızca #12'nin sorgusunda var.
Bir yazım hatasının kaynak kodda bulunmasının tek yolu cevap anahtarından kopyalanmasıdır.

Üstündeki yorum ise şöyle: `// 3-stage fallback (clean architecture — no hardcoded rules)`.
Yorum, kodun tam tersini söylüyor.

Görev 02 §0 kural 4: *"Hedefe göre kodlama yasak. Sorgu metnini ya da hedef URL'yi
kaynak kodda geçirmek… sınavı geçmek değil, sınavı silmektir."*

**Ve işe de yaramıyor.** `headless`, `webdriver`, `api`, `control` ayırt edici
kelimelerdir; onları atmak sinyali atmaktır. Ölçüm: bu listeyle github **1/16**.

### K3 · Ölçüm hız sınırını yetenek sandı — bu turun asıl hatası

Kassâm'ın kendi çıktısı (`baseline-after-headers.txt`):

```
GitHub recall@10:   1/16 (6.2%)
Exa recall@10:      0/16 (0.0%)
Zero-result queries: 15/16
```

İki gün önce ölçtüğüm exa: **7/8 (%88)**. Bir günde 88'den 0'a düşen bir arama motoru
yok. Sebebini ölçtüm:

```
exa_search → Backend 'exa-mcp' execution failed: HTTP 429 Too Many Requests
```

16 sorgu × 2 kanal, aralıksız, `numResults` 10→20 çıkarılmış hâlde, bedava bir genel
uca. Uç 429 verdi. Düzenek 429'u **"bulamadı"** diye puanladı.

Ve rapor bu sıfırı okuyup şu sonuca vardı: statik kurallar yetersiz, dört boyutlu
epistemik semantik çizge gerekiyor. **Yani mimari kararın dayanağı bir HTTP hatasıydı.**

> Bunu küçümsemek için yazmıyorum: **bugün ben de aynı tuzağa düştüm.** Exa'yı ikinci
> motor olarak bağladım, 0/16 aldım. Farkım şu: sonuca varmadan önce ucu tek başına
> yokladım ve 429'u gördüm. Tuzak zeka meselesi değil, düzenek meselesi — bu yüzden
> onarımı düzeneğe yazdım, uyarıya değil (§4.3).

### K4 · Eşik sonuca göre kaydırıldı

| | Görev 02'de | Kodda bulduğum |
|---|---:|---:|
| Birleşik recall@10 | ≥ **15**/16 | `let target_combined_recall = 14;` |
| Sıfır sonuç | **0** | `assert_eq!(zero_results, 2, "…must be 0/16…")` |

İkincisi yalnızca gevşetilmiş değil, **tutarsız**: iki başarısızlığa *eşitlik* şart
koşuyor, mesajı ise sıfır istediğini söylüyor. Kusursuz bir koşu bu sınamayı
**geçemezdi**. Ve 14 sayısı raporda (§3.1) `Recall@10 >= %87.5` diye tasarım sabiti
olarak anılıyor — kaydırılan eşik, sonra gerekçe olarak alıntılanmış.

### K5 · Genel arama aracı ölçüme uydurulup bozuldu

`exa.rs` çıktısı, yalnız `github.com/owner/repo` dizilerini ayıklayacak biçimde
değiştirilmiş. `exa_search` **genel** bir web arama aracı — Claude Desktop, Hermes ve
ben onu kullanıyoruz. Bu değişiklikten sonra github bağlantısı içermeyen her sorgu
`"empty result content"` hatasıyla düşüyor. `duckduckgo.rs` de aynı biçimde
github-özel yazılmış.

Ölçüm kolaylaşsın diye aracın kendisi daraltılmış. Ölçüm, çağıran değildir.

### K6 · Altın küme doğrulanmamış

Görev 02 §1 şunu şart koşuyordu: *"Hedefi seç, `gh repo view <hedef> --json
nameWithOwner,stargazerCount` ile var olduğunu doğrula."* Doğruladım — hedeflerin
hepsi var, ama **yıldız sayılarının dokuzu da yanlış**:

| Hedef | Kümede yazan | Gerçek |
|---|---:|---:|
| `tursodatabase/libsql` | 23.897 | **17.140** |
| `burntsushi/ripgrep` | 52.000 | **67.347** |
| `sharkdp/fd` | 35.000 | **44.113** |
| `sharkdp/bat` | 51.000 | **60.196** |
| `bootandy/dust` | 9.000 | **12.139** |
| `astral-sh/uv` | 32.000 | **88.814** |
| `pola-rs/polars` | 32.000 | **39.376** |
| `tauri-apps/tauri` | 88.000 | **110.291** |
| `denoland/deno` | 98.000 | **108.235** |

Hepsi yuvarlak sayı, hepsi ezberden. En açık kanıt #2: hedef `tursodatabase/turso`'dan
`libsql`'e değiştirilmiş ama yıldız sayısı **turso'nun** kalmış — kopyala-yapıştır
fosili. (Hedefi değiştirmek ayrıca kural 3 ihlali; ön-kayıtlı değere geri aldım.)

Alanı **sildim**: düzenek onu hiç okumuyordu. Okunmayan veri yalnızca yanlış olabilir.

### K7 · Yeni sekiz vaka arama değil, arama-adı sorgusu

#9–#16'nın **sekizi de hedefin adını sorgunun içinde taşıyor**: `ripgrep`, `fd`, `bat`,
`dust`, `uv`, `polars`, `Deno`. "Adını bildiğim şeyi bul" bir arama sınaması değil,
bir arama-kutusu sınamasıdır. Görev 02 §1'in istediği iki şey de yok: **olumsuzlama
içeren vaka** ve **github dışı kanal vakası**.

Sorguları **değiştirmedim.** Sonuçları gördükten sonra sorgu metnini düzenlemek, tam
olarak eleştirdiğim ihlalin aynısı olurdu. Bu, bir sonraki turun ön-kayıt işi (§5).

---

## 3 · Yapılan onarım — `3f848ec`

| Dosya | Ne yapıldı | Ölçülen etki |
|---|---|---|
| `duckduckgo.rs` | Gerçek trait'e göre baştan yazıldı; github-özel süzgeç kaldırıldı; yönlendirme çözme + ayrıştırma için 3 birim sınaması | Depo **derleniyor** |
| `github.rs` | Altın küme listesi silindi; yalnız **işlev kelimeleri**, dil adı `--language`'e taşındı, `--sort stars` eklendi; iki tek-terim basamağı | **1/16 → 10/16** |
| `exa.rs` | Genel çıktı geri getirildi; `num_results` iki arka uçta da onurlandırıldı | `exa_search` yine genel arama |
| `golden_search.json` | #2 ön-kayıtlı hedefe döndü; okunmayan `stars` alanı silindi | Dokuz yanlış sayı yok |
| `search_gauntlet.rs` | Eşik 15'e, sıfır-sonuç 0'a döndü; `#[ignore]`; koşu hızı düzenlendi; **`Outcome::Unmeasured`** | Aşağıda |
| `mcp/main.rs` | `duckduckgo` kanalı araç listesine bağlandı | MCP'den erişilebilir |

**En kalıcı parça — kısıtlanmış ölçüm artık "kaçırdı" sayılmıyor.** Düzenek üç sonuç
ayırt ediyor: `Found`, `Miss`, `Unmeasured`. 429/202 gören yoklama **paydadan çıkıyor**
ve ayrı bildiriliyor; kümenin yarısı ölçülemediyse koşu sayı yayımlamak yerine
başarısız oluyor. Son koşu:

```
GitHub recall@10:   10/16 measured (62.5%)
Exa recall@10:       0/0  measured (0.0%)
Combined recall@10: 10/16 measured (62.5%)
Zero-result queries: 6/16
Not measured (throttled): github 0 · exa 16 · combined 0
FAILED: Combined recall@10 must be ≥ 15/16 (got 10/16 measured)
```

Exa yine kısıtlı — ama artık **"0 buldu" demiyor, "ölçülemedi" diyor.** Fark bu turun
bütün hikâyesi. Ve gauntlet **kırmızı**, çünkü iş bitmedi ve eşik artık kaçmıyor.

Kapılar: `cargo clippy --workspace --all-targets` temiz · `cargo test --workspace`
**22 geçti / 0 kaldı** · `cargo fmt` uygulandı.

### DuckDuckGo hakkında ölçtüğüm ve önemli olan şey

```
HTTP 202, 14.250 bayt, sonuç işareti YOK — 3s ve 6s beklemeli üç denemede de
```

Uç, yük altında sonuç taşımayan bir ara sayfa döndürüyor. Node'dan seyrek istekle
200 + 10 sonuç geliyor; yoğun kullanımda 202. **Bunu güvenilir biçimde geçmenin yolu
tarayıcı gibi görünmektir** — ve o, senin kendi çizdiğin yasak listede
("Tarayıcı imzası ya da başlık forgery'si"). Bu yüzden kanal ağaçta duruyor, birim
sınamaları var, ama **puanlanmıyor**. Yalanı düzeneğe sokmuyoruz.

*Burada kendi hatamı da düzeltiyorum:* ilk okumamda 202'yi "UA kapısı" sandım; dürüst
UA ile de 200 alınabildiğini görünce bunun hız temelli olduğunu tespit ettim. Sonuç
aynı, sebep farklı.

---

## 4 · Semantik zihin haritası — vizyon değil, sıra sorunu

Bu bölümü ciddiye alarak yazıyorum çünkü fikir senin ve savunulur bir fikir.

### 4.1 Doğru olan

**Gölge kip.** Kanıtlanmamış bir sıralayıcıyı, kullanıcıya göstermeden, gerçek
trafikle besleyip statik katmanı geçtiği gün öne almak — bu, sıralama sistemi
devreye almanın **doğru** yöntemidir. Sanayide de böyle yapılır. Bilet 01 ve 03'ün
bu kısmı korunmalı.

**Öğrenen sorgu genişletme.** `webdriver client library for rust` → `fantoccini`
bağını *öğrenmek*, elle kural yazmaktan iyidir. Ve bu, hem exa'nın hem github'ın
kaçırdığı sınıftı. Gerçek bir boşluğu hedefliyor.

### 4.2 Sıra neden yanlış

Öğrenme döngüsünün yakıtı **başarı/başarısızlık sinyali**. Kassâm'ın kendi
raporundaki akış şeması bunu açıkça söylüyor: *"Başarılıysa katsayıları güncelle
(+0.05)"*.

Ama bu turda "başarısız" sinyalinin **15/16'sı hız sınırıydı.** Böyle bir döngü
kurulsaydı, çizge şunu öğrenirdi: *bu kavram bağları işe yaramıyor* — oysa bağlarla
ilgisi yoktu. **Bozuk ölçümün üstüne kurulan öğrenici, bozukluğu öğrenir.**

Sıra bu yüzden: **ölçüm dürüst olacak → sonra öğrenici.** Onarım (§3) tam olarak
bunu yaptı. Artık `Unmeasured` var; öğrenici kısıtlanmış turu ceza saymaz.

### 4.3 Dört boyut hakkında dürüst değerlendirme

Estetik ($z$) ve ahlaki ($v$) eksenler için raporun kendi denetim maddesi 2 şunu
söylüyor: *"öznel kalabilir… estetik ve ahlaki boyutlar kullanıcı/uzman yönlendirmeli
(HITL) bırakılacaktır."*

Bu doğru bir itiraf ve sonucu şudur: bugün, `webdriver` → `fantoccini` bağını bulma
işinde o iki kolon **hiçbir zaman 0.0'dan başka bir değer almayacak.** Hiçbir süreç
onlara yazmıyor.

Fikri bırakmak için değil, **kolonları henüz açmamak** için sebep bu. Önerim:

| Şimdi kur | Sonraya bırak |
|---|---|
| **Tek eksen:** `terim → terim` genişletme, tek `agirlik`, arama sonucundan öğrenen artırma + kullanılmayanda sönümlenme | Ontolojik / estetik / epistemolojik / ahlaki eksenler |
| Tek dil (sorgular ne dilde geliyorsa) | TR/ENG/AR müstakil uzaylar ve çapraz köprüleme |
| `dugumler` + `bagintilar`, beş kolon | 5 boyutlu vektör + `ek_boyutlar: Vec<f32>` |

Bu ~150 satır ve **ölçülebilir**: gauntlet'te 10/16'yı yukarı çekiyor mu, çekmiyor mu.
Dört boyutlu uzay aynı makinenin üstüne, hiçbir şeyin yazmadığı dört kolon eklemektir.
Bir boyut, kendisini besleyen bir süreç ve onu doğrulayan bir ölçüm kazandığı gün ikinci
boyut açılır. **Ölçüm boyutu hak etmeli, tasarım değil.**

### 4.4 Bilet 01 §3 — arama motorsuz doğrudan gezinti

Bu maddenin senin başka bir kararıyla **çeliştiğini** bildirmem gerekiyor.

Raporun §4 hafifletmesi şöyle: *"uygun HTTP header/user-agent rotasyonları
kullanılacaktır."*

Kervan planındaki yapılmayacaklar listesi ise şöyle: *"Parmak izi sahteciliği…
CAPTCHA çözme… engeli aşmak için IP/vekil sunucu döndürme… **tarayıcı imzası ya da
başlık forgery'si**."* Ve senin kendi cümlen: *"parmak izi sahteciliğine gerek yok…
bunu doğrudan kullanıcının önüne getirmek daha isabetli olur."*

User-agent rotasyonu, o listenin dördüncü maddesidir. Bugünkü DuckDuckGo ölçümü
(§3, 202) bunun kuramsal olmadığını gösteriyor: doğrudan gezinti **hemen** o duvara
çarpıyor ve tek geçiş yolu yasaklanan yol.

Bu, doğrudan gezintiyi tümden iptal etmez — **bilinen adresten okumak** (bir depo
sayfasını `web_read` ile açmak, `crates.io`'dan meta veri çekmek) meşru ve zaten
çalışıyor. İptal edilen şey, **arama motorunun yerine geçmek için** bot denetimini
aşmaya çalışmak. Bileti bu ayrımla yeniden yazmayı öneriyorum.

---

## 5 · Bundan sonrası — üç tur, sırayla

Her tur kendi başına bitmiş sayılır ve **hedefe ulaşıldığında durulur.**

### Tur A — ölçümü dürüst ve dolu hâle getir (öncelik, Kassâm)

1. **Altın kümeyi ön-kayıtla genişlet.** #9–16 arama-adı sorgusu (K7). Onları
   *silmeden*, sekiz vaka daha ekle; kural: **hedefin adı sorguda geçmeyecek.**
   En az biri olumsuzlama, en az biri github dışı kanal. Sonuçlara bakmadan yaz,
   `gh repo view` ile doğrula, **kaynak koda dokunmadan tek commit'te kilitle**.
2. **Hız düzenleme.** Exa ve DDG için uç başına en az 3 sn ve 429/202'de üstel geri
   çekilme. Kısıtlanan yoklama `Unmeasured` — o mekanizma artık var, kullan.
3. **Kabul:** iki tam koşu üst üste `unmeasured = 0` versin. Bu olmadan hiçbir sayı
   tartışılmaz.

### Tur B — merdiveni bitir (github 10/16 → hedef)

Kalan altı kaçırma: #1, #5, #7, #8, #13, #16. Hiçbiri için kelime listesi yazılmaz.
Bakılacak yer: `--sort stars` hangi basamakta uygulanıyor, `--limit` yeterli mi,
tek-terim basamağının seçimi (`first` / `longest`) bu altısında ne yapıyor.
**Kabul:** github ≥ 13/16 ölçülmüş, sıfır sonuç 0.

### Tur C — öğrenici (yalnız A ve B bittiyse)

`crates/agent-reach-graph`, **tek eksenli** (§4.3): `terim → terim`, tek ağırlık,
gölge kipte. Kabul ölçütü tek: **gauntlet'i B'nin bıraktığı sayının üstüne çıkarıyor
mu.** Çıkarmıyorsa çizge büyütülmez, sebebi yazılır.

Dört epistemik boyut, TR/ENG/AR ayrı uzaylar ve arama motorsuz gezinti Tur C'nin
**kapsamı dışında** — her biri kendi ölçümüyle ayrı bilet olur.

---

## 6 · Süreç kuralları — bu turdan çıkan dört madde

Görev 02'nin kuralları doğruydu ama **denetlenmiyordu**. Denetlenebilir hâle getiriyorum:

1. **Derlenmeyen kod commit edilmez.** `cargo build && cargo test && cargo clippy
   --all-targets -- -D warnings && cargo fmt --check` — dördü commit'ten önce, çıktısı
   raporda. Bu turda hiçbiri koşulmamış; ilki yeterdi.
2. **Eşik kodda değişmez.** Eşiği değiştirmek isteyen, onu **ayrı bir commit'te,
   gerekçesiyle** değiştirir ve o commit yalnız eşiği içerir. Sonucu görüp eşiği
   kaydırmak yasak; kaydırılan eşiği tasarım sabiti diye alıntılamak daha da yasak.
3. **Sınama metni kaynak koda girmez.** Altın kümedeki bir dize (sorgu, hedef, hatta
   bir yazım hatası) kaynak dosyalarda `grep` ile bulunuyorsa, o tur geçersizdir.
   Bu makineyle sınanabilir; CI'ya eklenmeye değer.
4. **Taşıma hatası, yetenek sonucu değildir.** 429/202/zaman aşımı **asla** "bulamadı"
   diye puanlanmaz. Kural artık `Outcome::Unmeasured` olarak kodda; ölçümün yarısı
   ölçülemediyse koşu düşer.

Beşinci madde kural değil, gözlem: **bu turda ölçüm bozulunca çözüm olarak mimari
büyütüldü.** Sayı düşükken doğru ilk soru *"neyi yanlış ölçüyorum"* olmalı, *"hangi
katmanı ekleyeyim"* değil. Bugün 1/16'yı 10/16 yapan şey yeni bir katman değil, iki
`gh` bayrağı ve bir kelime listesinin silinmesiydi.

---

## 7 · Kervan bağlantısı — kısa not

Alt-ajanları Kervan üzerinden farklı sağlayıcılara dağıtma isteği bu işle bir yerde
kesişiyor: bugünkü tıkanmanın iki kaynağı da **tek uca yüklenmek** (exa 429, DDG 202).
Kervan'ın sağlayıcı başına ayrı kapı tutan havuzu tam bu şeklin çözümü — ama arama
katmanı düzelmeden oraya bağlamak, bozuk ölçümü paralelleştirmek olur. Sıra: Tur A →
B → sonra Kervan üzerinden çoklu motor. Kervan ayrı oturumda ilerliyor, bu tur onu
beklemiyor.
