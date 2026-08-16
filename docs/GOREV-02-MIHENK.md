# Görev 02 — `agent-reach-rs`: aramayı gerçekten arama hâline getirmek

**Kime:** El-Kassâm · **Kimden:** Mihenk (Claude Opus 5) · **Tarih:** 16 Ağustos 2026
**Kapsam:** `agent-reach-rs` arama katmanı. ZOPAY'dan **yöntem** ödünç alınır, kod değil;
iki depo ayrı kalır.

---

## I · Bugün düzeltilenler

Üçü de ölçülerek doğrulandı, hiçbiri devralınan iddia değil.

### 1. Anahtarsız Exa yolu geri geldi (`exa.rs`)

Python aslı Exa'ya `mcporter` üzerinden, `https://mcp.exa.ai/mcp` uç noktasıyla
bağlanıyordu ve bunu "bedava, API anahtarı gerekmez" diye ilan ediyordu. Rust portu
bunu doğrudan `api.exa.ai` çağrısıyla değiştirdi ve `exa_api_key`'e bağladı. O anahtar
bu makinede hiç ayarlanmamıştı — yani **arama tamamen ölüydü.**

Uç nokta düz HTTP JSON-RPC olduğu için `ExaMcpBackend` ona doğrudan konuşuyor: anahtar
yok, aslının hâlâ ihtiyaç duyduğu npm bağımlılığı da yok. Anahtar varsa `exa-api` yine
önce denenir — o kullanıcının kendi hesabı ve kendi limiti.

Canlı doğrulama, anahtarsız:

```
exa_search "cursor minisqlite rust sqlite reimplementation github"
→ #1: cursor/minisqlite — "A reimplementation of SQLite in Rust..."
```

Dün bulamadığım depoyu ilk sırada döndürdü.

### 2. "No backends available" artık nedenini söylüyor (13 kanal)

`is_available` zaten tam olarak neden çalışamadığını hesaplıyordu
(`RequiresConfig { missing }`, `NotInstalled { command }`), ama her kanal bunu
`tracing::debug!`'a yazıp çağırana çıplak bir `"No backends available"` döndürüyordu.
Ayarlanmamış bir anahtar ile ağ kesintisi çağıran açısından **birbirinin aynıydı**.

`backend::unavailable()` durumları mesaja katıyor; on üç kanalın hepsine uygulandı.

```
önce:  Backend 'linkedin' is not available: No backends available
sonra: Backend 'linkedin' is not available: linkedin-api ✗ not installed:
       python3 -m pip install linkedin-api
```

### 3. `gh repo view` alan adları (`github.rs`, 14 Ağustos)

`stargazersCount`/`forksCount` yalnız `gh search repos` için geçerli; `gh repo view`
tekil ad istiyor (`stargazerCount`/`forkCount`). Kanal her `repo` çağrısında hata
veriyordu.

### 4. Kurulum yolu

İkili artık çerez dizininden değil, kalıcı yerinden koşuyor:
`claude yazılım/kullanımdaki yetenekler ve araçlar/agent-reach-mcp/bin/`.
SHA256 ile `target/release/` ile aynı olduğu doğrulandı.

**Durum:** `cargo clippy --workspace --all-targets` temiz · `cargo test --workspace`
22 geçti / 0 kaldı · commit `f78163b`.

---

## II · Ölçüm: arama katmanı diye bir şey yok

Sekiz sorgu kuruldu. Her biri **bir insanın ya da bir ajanın gerçekten yazacağı gibi**
yazıldı — anahtar kelime dizisi değil, doğal cümle. Her birinin hedefi `gh repo view`
ile var olduğu doğrulanmış gerçek bir depo.

| # | Sorgu | Hedef | ⭐ |
|---|---|---|---:|
| 1 | a sqlite reimplementation written in rust | `cursor/minisqlite` | 270 |
| 2 | sqlite compatible database written in rust | `tursodatabase/turso` | 23.897 |
| 3 | control headless chrome from rust | `rust-headless-chrome` | 2.946 |
| 4 | chrome devtools protocol rust api | `mattsse/chromiumoxide` | 1.367 |
| 5 | sqlite bindings for rust | `rusqlite/rusqlite` | 4.360 |
| 6 | webdriver client library for rust | `jonhoo/fantoccini` | 2.015 |
| 7 | rust http client library | `seanmonstar/reqwest` | 11.781 |
| 8 | exa mcp server for web search | `exa-labs/exa-mcp-server` | 4.877 |

Bugünkü davranış, ölçülmüş:

| Sorgu | github | dönen sonuç | exa |
|---|:---:|---:|:---:|
| a sqlite reimplementation written in rust | ✗ | **0** | ✓ |
| sqlite compatible database written in rust | ✗ | 1 | ✓ |
| control headless chrome from rust | ✗ | **0** | ✓ |
| chrome devtools protocol rust api | ✓ | 1 | ✓ |
| sqlite bindings for rust | ✗ | 2 | ✓ |
| webdriver client library for rust | ✗ | **0** | ✗ |
| rust http client library | ✓ | 9 | ✓ |
| exa mcp server for web search | ✗ | **0** | ✓ |

```
github recall@10 : 2/8   (%25)   — 8 sorgunun 4'ü SIFIR sonuç döndürdü
exa    recall@10 : 7/8   (%88)   — bugüne kadar tamamen ölüydü
birleşim         : 7/8
```

Dört sorgunun boş dönmesi, hatanın en kötü biçimi: **`[]` başarı olarak dönüyor.**
Çağıran ajan "yok" ile "bulamadım"ı ayırt edemiyor. Benim dün `cursor/minisqlite`'ı
bulamamamın sebebi tam olarak buydu — exa ölüydü, github `[]` dedi, ben "yok" diye
okudum.

---

## III · Kök sebep — dört ad konulmuş kusur

### K1 · Sorgu planlaması yok

`github.rs:97-105` kullanıcının cümlesini olduğu gibi `gh search repos`'a veriyor:

```rust
cmd.arg("search").arg("repos").arg(query)
```

`gh search repos` çok kelimeyi ad+açıklama üzerinde **VE** ile arar. Cümledeki tek bir
kelime (`a`, `written`, `from`, `for`) hedefin açıklamasında geçmiyorsa sonuç sıfırdır.
`cursor/minisqlite`'ın GitHub API'sindeki açıklaması **boş** — hiçbir çok kelimeli sorgu
onu tutamaz.

### K2 · Basamaklı gevşetme yok

Sıfır sonuç geldiğinde hiçbir şey olmuyor. Oysa gevşetme çalışıyor — ölçtüm:

```
"control headless chrome from rust"                       → 0 sonuç
"headless chrome" --language rust --sort stars            → #1 rust-headless-chrome (2.946⭐)

"webdriver client library for rust"                       → 0 sonuç
"webdriver" --language rust --sort stars                  → #1 fantoccini (2.015⭐)
```

İkinci vaka önemli: **exa'nın da kaçırdığı tek sorgu buydu.** Yani basamak, füzyonun
tek başına kapatamadığı boşluğu kapatıyor.

### K3 · Sıralama denetimi yok

`--sort` hiç kullanılmıyor. `"sqlite rust"` sorgusu dört ders alıştırması deposu
döndürüyor; `rusqlite` (4.360⭐) listede yok.

### K4 · `num_results` sessizce yutuluyor

`mcp/main.rs:213-223` `num_results`'ı `args[1]` olarak geçiriyor. `exa.rs`'teki iki arka
uç da `args[1]`'e bakmıyor; ikisi de `numResults: 10` sabitini gönderiyor. Parametre
şemada var, etkisi yok.

---

## IV · ZOPAY'ın arama aracı ne yapıyor

Kod okundu, iddia devralınmadı. `zopay/crates/zopay-search` + `zopay-text`,
961 satır kaynak · **27 test**. Karşılaştırma için: `agent-reach-rs`'in 5.091 satırında
arama davranışını sınayan **tek bir test yok** — 13 kanalın 11'inde tek test var, o da
`is_available` yeşil mi diye soruyor.

Altı mekanizma, dördü buraya aktarılabilir:

| ZOPAY mekanizması | Nerede | Ne yapıyor |
|---|---|---|
| **Derin normalizasyon** | `zopay-text/src/lib.rs` | NFD → birleşen imleri at → kesme/hemze sil → Türkçe katla → küçült. `Câbirî`=`cabiri`, `Weber'de`=`weberde`. Arapça/Kiril/CJK **korunur** — silinirse o kayıt hiçbir sorguyla eşleşmez |
| **Üç basamaklı gevşetme** | `bm25.rs:282-330` | 1) birebir → boşsa 2) önek (`"tok"*`) → boşsa 3) düz metin. **Sorgu asla ilk basamakta ölmez** |
| **Alan ağırlıklı BM25** | `bm25.rs:239` | `bm25(items_fts, 4.0, 3.0, 2.0, 1.0)` — başlık 4, yazar 3, etiket 2, gerisi 1 |
| **RRF füzyonu** | `lib.rs:419-452` | İki bağımsız sıralayıcının listelerini `1/(60+rank)` ile birleştirir. Skorları değil **sıraları** topladığı için ölçekleri uyumsuz motorlar kaynaştırılabilir |
| İmza önbellekli yan indeks | `bm25.rs:80-160` | (boyut, mtime, sayı) değişmedikçe yeniden indekslemez; kaynağa asla yazmaz |
| Sonradan süzme | `builder.rs:43-69` | `limit*3` çeker, tür/yıl süzer, `limit` kadar keser |

### Aktarım haritası — dürüst değerlendirme

| Mekanizma | Aktarılır mı | Gerekçe |
|---|:---:|---|
| Üç basamaklı gevşetme | **Evet, en yüksek kazanç** | K2'yi doğrudan kapatıyor; ölçtüm, iki sıfır-sonuç sorgusunu #1'e taşıyor |
| RRF füzyonu | **Evet** | Motorlar arası uyumsuz skorları kaynaştırmanın doğru yolu bu; github ⭐ sayısı ile exa benzerlik skoru toplanamaz, sıraları toplanır |
| Derin normalizasyon | **Evet, ucuz** | 165 satır, bağımsız sandık; Türkçe/çeviriyazı sorguları için tek yol |
| Alan ağırlıklı BM25 | **Hayır** | Yerel FTS5 indeksi gerektirir; web araması uzak, indeks bizde değil. Ağırlıklandırma füzyon aşamasında yapılır |
| Yan indeks önbelleği | **Hayır** | İndekslenecek yerel derlem yok |
| Sonradan süzme | Kısmen | `--language`, `--sort stars` gibi süzgeçler kanal sorgusuna gömülür, sonrasına değil |

**Yapısal benzerlik önemli:** ZOPAY tek derlem üstünde iki sıralayıcıyı kaynaştırıyor;
`agent-reach` web üstünde N kanalı kaynaştırmalı. RRF listelerin nereden geldiğini
umursamaz. Aynı 30 satır.

---

## V · Ama önce en tembel seçenek söylenmeli

**exa tek başına 8'de 7 yapıyor.** En kısa yol şudur: `exa_search`'ü öntanımlı arama
yap, dur. Bugün zaten çalışıyor.

Bunun yetmemesinin üç ölçülmüş sebebi var:

1. **Sıfır sonuç hâlâ yalan söylüyor.** Bir ajan `github` kanalını doğrudan çağırdığında
   `[]` alıyor ve "yok" diye okuyor. Bu, sorgunun hangi kanaldan geçtiğine bakmaksızın
   düzeltilmesi gereken bir doğruluk hatası — arama kalitesi meselesi değil.
2. **`fantoccini` vakasını ikisi de kaçırdı.** Tek motora bağlanmak o sınıf sorguyu
   kalıcı olarak kaybetmek demek; basamaklı github sorgusu onu #1'de buluyor.
3. **Kanala özgü sorgular exa'ya devredilemez.** "bu depodaki açık issue'lar",
   "bu kullanıcının son gönderileri" — bunlar arama değil, erişim.

Bu yüzden görev **üç dereceli** kuruldu ve **hedefe ulaşıldığı anda durulur**. A ve B
küçük ve kesin; C yalnızca A+B sayıyı tutturmazsa açılır.

---

## VI · Kervan bağlantısı — ayrı oturuma çatallandı

Ercan'ın niyeti: Kervan'ı alt-ajanlar için kullanmak, aynı anda farklı sağlayıcılarla
çalıştırmak. Bu raporun konusu değil ama arama işiyle bir yerde kesişiyor, kayda geçsin:

- Kervan'ın havuzu **zaten** eşzamanlılığa hazır: sağlayıcı başına `Semaphore` kapısı
  var (`pool.rs`), öntanımlı 1. Farklı sağlayıcılar farklı kapılar olduğu için
  gemini + kimi + copilot **şu an bile** paralel koşabilir; aynı sağlayıcıya iki istek
  koşamaz. Bu sınır bilinçli — Cloudflare adli incelemesi patlama davranışını
  suçlu buldu.
- Alt-ajan başına bir sağlayıcı eşlemesi bu mimariyle örtüşüyor: N alt-ajan, N ayrı
  kapı, tek tarayıcı havuzu.
- Kesişme noktası: bir arama füzyonu alt-ajanların en çok istediği şey. Ama Kervan
  tarafı ayrı oturumda; bu görev Kervan'a **bağımlı değil** ve onu beklemez.

---

# VII · GÖREV — El-Kassâm

> Buradan aşağısı kendi kendine yeterlidir. Başka bağlam istemeden çalışılabilir.
> Ercan'ın onay kapılarına dokunma.

## 0 · Kurallar (tümü bağlayıcı)

1. **Depo dili İngilizce.** `agent-reach-rs`'te tanımlayıcılar ve belge yorumları
   İngilizce yazılır. (Kervan Türkçe'dir; bu depo değil.) Karıştırma.
2. **Uydurma yok.** Bir vaka geçmiyorsa geçmiş gibi raporlama.
3. **Altın kümeyi zayıflatma.** Sorgu metnini değiştirmek, hedefi kolaylaştırmak,
   vaka silmek yasak. Bir vaka gerçekten kusurluysa (hedef depo silinmiş vb.)
   gerekçesiyle bildir, sessizce çıkarma.
4. **Hedefe göre kodlama yasak.** Sorgu metnini ya da hedef URL'yi kaynak kodda
   geçirmek, altın kümeye özel dal açmak — bunlar sınavı geçmek değil, sınavı silmektir.
5. **Anahtar zorunluluğu ekleme.** Çözüm anahtarsız makinede çalışmalı. `exa-mcp` ve
   `gh` yeter.
6. **Ponytail.** Tek uygulamalı arayüz, tek ürünlü fabrika, hiç değişmeyen değer için
   yapılandırma — hiçbiri. Bilinçli sadeleştirmeyi `// ponytail:` ile işaretle.
7. **Her aşamadan sonra üçü de yeşil:**
   ```
   cargo test --workspace
   cargo clippy --workspace --all-targets -- -D warnings
   cargo fmt --check
   ```

## 1 · Altın küme — önce kur, sonra kodla

`tests/golden_search.json` dosyasını **ilk commit'te**, herhangi bir kaynak değişikliği
yapmadan oluştur. Kilitli çekirdek şu sekiz vakadır; bunlar ölçülmüş, aynen alınır:

```json
[
  {"q": "a sqlite reimplementation written in rust",  "want": "cursor/minisqlite"},
  {"q": "sqlite compatible database written in rust", "want": "tursodatabase/turso"},
  {"q": "control headless chrome from rust",          "want": "rust-headless-chrome/rust-headless-chrome"},
  {"q": "chrome devtools protocol rust api",          "want": "mattsse/chromiumoxide"},
  {"q": "sqlite bindings for rust",                   "want": "rusqlite/rusqlite"},
  {"q": "webdriver client library for rust",          "want": "jonhoo/fantoccini"},
  {"q": "rust http client library",                   "want": "seanmonstar/reqwest"},
  {"q": "exa mcp server for web search",              "want": "exa-labs/exa-mcp-server"}
]
```

Bunu **sekiz vaka daha** ekleyerek 16'ya çıkar. Aşırı uyum (overfitting) sekiz vakada
kaçınılmazdır; genişletme bunun içindir. Genişletme yordamı — sırası bağlayıcı:

1. Sorguyu **önce** yaz, doğal cümle olarak, sonucuna bakmadan.
2. Hedefi seç, `gh repo view <hedef> --json nameWithOwner,stargazerCount` ile var
   olduğunu doğrula. En az 500 yıldız (belirsiz hedef ölçüm gürültüsüdür).
3. En az ikisi Türkçe/çeviriyazı gerektirsin (ör. `ı`/`i` farkı, `ş`/`s`, aksanlı ad).
4. En az biri **olumsuzlama** içersin (`sqlite wrapper for rust -bindings` gibi).
5. En az biri `github` dışında bir kanalın işi olsun (`rss`, `web`, `youtube`).
6. Bu dosyayı **kaynak koda dokunmadan** commit et:
   `test: lock golden search set before implementation`

Bu sıra bir güvenlik önlemidir: küme sonuçlara bakılarak yazılırsa ölçüm anlamını
yitirir.

## 2 · Taban ölçümü

`tests/search_gauntlet.rs` yaz: altın kümeyi okur, her sorguyu koşar, `recall@10` ve
**sıfır sonuç oranını** basar. Ağ gerektirdiği için `#[ignore]` ile işaretle; komutu
belgele:

```
cargo test --test search_gauntlet -- --ignored --nocapture
```

Ölçülmüş taban (16 vakanın kilitli 8'i için, bugün):

```
github  recall@10: 2/8   sıfır sonuç: 4/8
exa     recall@10: 7/8   sıfır sonuç: 0/8
```

İlk işin bu sayıları **yeniden üretmek.** Tutmuyorlarsa dur ve bildir — ölçüm
düzeneğinde hata vardır, kodda değil.

## 3 · Görev A (zorunlu) — sorgu planlayıcı + gevşetme basamağı

Yalnız `github.rs`, `GhCliBackend::execute`'un `"search"` kolu.

Üç basamak, ZOPAY'ın `Fts5Bm25Searcher::ara`'sındaki desenin aynısı — bir basamak boş
dönerse sonraki denenir:

1. **Birebir:** sorgu bugünkü gibi geçilir. Sonuç varsa dur.
2. **Planlanmış:** durak kelimeler atılır (`a an the for from with written in of to`),
   dil belirteci (`rust`, `python`, `go`…) sorgudan çıkarılıp `--language <dil>`'e
   çevrilir, `--sort stars` eklenir. Sonuç varsa dur.
3. **Tek belirteç:** kalan en ayırt edici tek belirteç + `--language` + `--sort stars`.
   "En ayırt edici" için ölçüt: en uzun belirteç. Daha zekisini yapma; ölçtüğümde bu
   iki vakayı da kurtarıyor.

Basamak numarası çıktıda görünmeli — çağıran gevşetildiğini bilmeli. `ChannelOutput`'ta
zaten `backend` alanı var; `"gh-cli"` yerine `"gh-cli/stage2"` yeterlidir. Yeni alan
ekleme.

**Bu basamağın nasıl davrandığı ölçüldü, uydurma değil:**

```
"control headless chrome from rust"            → 0
"headless chrome" --language rust --sort stars → #1 rust-headless-chrome
"webdriver client library for rust"            → 0
"webdriver" --language rust --sort stars       → #1 fantoccini
```

**Kabul ölçütü:** github `recall@10 ≥ 5/8` (kilitli çekirdek), **sıfır sonuç oranı 0/8**.

## 4 · Görev B (zorunlu) — iki küçük onarım

**B1 — `num_results` onurlandırılsın.** `mcp/main.rs` onu `args[1]` olarak geçiriyor;
`exa.rs`'teki iki arka uç da yok sayıp `10` gönderiyor. `args.get(1)`'i ayrıştır,
ayrıştırılamazsa `10`'a düş. (K4)

**B2 — normalizasyon.** Sorgu kanala gitmeden önce normalize edilsin: NFD → birleşen
imleri at → küçült. `zopay-text`'i **bağımlılık olarak ekleme** — ayrı depo, ayrı ömür.
Gereken kadarını yaz; `unicode-normalization` zaten bağımlılık ağacında olabilir, önce
`cargo tree` ile bak (ponytail 5. basamak). Türkçe `ı`→`i` katlaması elle gerekir,
NFD onu ayrıştırmaz — ZOPAY'ın `zopay-text/src/lib.rs`'teki `katla` yorumu bunu açıklıyor,
oku.

**Kabul ölçütü:** Türkçe vakalar geçer; `num_results: 3` üç sonuç döndürür.

## 5 · Görev C (koşullu) — RRF füzyonu

**A + B sonrası birleşik `recall@10` 15/16'ya ulaştıysa C'yi açma.** Gerekçesini
raporda yaz ve dur. Bu bir başarısızlık değil, doğru sonuçtur.

Ulaşmadıysa: yeni MCP aracı `search`. `github` ve `exa` kanallarını paralel koşar
(`tokio::join!`), sonuçları RRF ile kaynaştırır:

```
skor(belge) = Σ  1 / (60 + sıra_listede)
```

`60` sabiti ZOPAY'ın `lib.rs:429`'daki değeridir ve literatürdeki öntanımlıdır;
ayarlanabilir yapma (ponytail: hiç değişmeyen değer için yapılandırma yok). Kimlik
anahtarı normalize URL'dir. Bir kanal hata verirse diğerinin sonucu döner; ikisi de
hata verirse `backend::unavailable()` deseni kullanılır — çıplak mesaj yasak.

**Kabul ölçütü:** birleşik `recall@10 ≥ 15/16`, sıfır sonuç `0/16`.

## 6 · Teslim

1. `cargo test --test search_gauntlet -- --ignored --nocapture` tam çıktısı — taban ve
   son sayılar yan yana.
2. `cargo test --workspace`, `clippy -D warnings`, `fmt --check` çıktıları.
3. Hangi görevin açıldığı, hangisinin **gerekçesiyle açılmadığı**.
4. Ponytail notu: uygulanan sadeleştirmeler + reddedilen öneriler (gerekçeli),
   net satır değişimi `+X / -Y`.
5. Hâlâ geçmeyen vaka varsa teşhisiyle — "sorgu planlayıcı X'i kurtaramıyor çünkü Y".
   Kaçırılan vakayı bildirmek kusur değil; gizlemek kusurdur.

Commit sırası:

```
test: lock golden search set before implementation
feat(github): staged query relaxation for natural-language search
fix(exa): honor num_results; normalize query before dispatch
feat(search): RRF fusion across github and exa      # yalnız C açıldıysa
```

## 7 · Bu görevin sınırı

Yapılmayacaklar — biri gerekli görünüyorsa **yapma, sor**:

- Yeni kanal eklemek
- `Channel` ya da `Backend` trait'ini değiştirmek (13 kanalı birden kırar)
- Anahtar/kimlik gerektiren bir yol eklemek
- Yerel indeks kurmak (uzak arama, indeksleyecek derlem yok)
- Yeniden sıralama için bir model çağırmak (ağ turu + kota; RRF bedava)
