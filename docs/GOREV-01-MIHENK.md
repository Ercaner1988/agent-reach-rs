# Görev 01 — `agent-reach-rs`: kapıyı işe bağlamak

**Kime:** El-Kassâm · **Kimden:** Mihenk (Claude Opus 5) · **Tarih:** 12 Ağustos 2026
**Kapsam:** `agent-reach-rs` — ZOPAY'dan bağımsız araç, ZOPAY deposuna katılmaz.

---

## I · Önce ölçüm — ve bu iyi bir tablo

Denetim koşuldu, devralınan sayı yok (altıncı kural):

| | |
|---|---:|
| Commit | 23 · 9–11 Ağustos, üç gün |
| Rust satırı | 4.854 · 29 dosya · 4 sandık |
| Kanal | **13**, her biri çift arka uçlu (CLI aracı + REST/web) |
| `cargo test --workspace` | **20 geçti / 0 kaldı** |
| `cargo clippy --all-targets -- -D warnings` | temiz |
| `cargo fmt --check` | temiz |

Ayrıca: GitHub Actions CI kurulu ve doğru şeyleri koşuyor (build · test · fmt ·
clippy), `cargo-dist` ile çok platformlu sürüm yapılandırılmış, README üç dilde
(TR/EN/AR) ve senkron tutulmuş, commit'ler düzenli — kanal başına bir commit.

**Üç günde çıkan iş olarak bu iyi ve söylenmesi gerekiyor.** Aşağıdaki maddeler
işin kötü olduğunu değil, **kapının yanlış yeri ölçtüğünü** söylüyor.

---

## II · Bulgu: 13 kanal var, sınanan 1 tanesi

Sınamaların dağılımı:

| Dosya | Sınama |
|---|---:|
| `channels/rss.rs` | 4 |
| `core/config.rs` | 3 |
| `channels/web.rs` | 2 |
| **diğer 10 kanal** | **her biri 1** |

O "her biri 1"in hepsi aynı şeyi deniyor:

```rust
let status = backend.is_available(&config).await;
assert!(matches!(status, BackendStatus::Available));
```

Bu, **"yapılandırılmış mı"** sorusunu soruyor. Kanalın var olma sebebi olan soruyu
— *veri çekiyor mu, doğru ayrıştırıyor mu* — hiçbiri sormuyor. `is_available`
yeşilken `fetch` tamamen bozuk olabilir ve kapı susar.

Sekizinci kural: yeşil bir kapı kapsamadığı şey hakkında sessizdir. Burada sessiz
kaldığı şey aracın kendisi.

Davranış sınayan tek dosya `web.rs` (`test_web_channel_actions`).

### Ve bir tanesi hiç düşemiyor

`channels/linkedin.rs`:

```rust
match status {
    BackendStatus::RequiresConfig { .. } => {}
    BackendStatus::NotInstalled { .. } => {}
    _ => panic!("Expected RequiresConfig or NotInstalled, got {:?}", status),
}
```

Yapılandırma yokken ulaşılabilecek iki durum bunlar, ve ikisi de kabul ediliyor.
Dördüncü kural: **düşmeyen kapı ölçmez.** Bu sınama silinse CI'nin rengi değişmez.

### İyi haber: eksik kanıt zaten elimizde

Kökteki `verification_log.json`, `test_execution_log.json`, `rss_execution_log.json`
gerçek koşu kayıtları — canlı çağrılar, kanal/arka uç/süre/başarı alanlarıyla.
Yani kanallar **elle uçtan uca denenmiş ve çalışmış.**

Ama bu kayıtlar bir *kere* olanı anlatıyor; yedinci kural: bir aracın çıktısındaki
sayı, yaptığı işin sürekli kanıtı değildir. CI o kanıtı üretmiyor.

**İşin özü şu: yapılacak şey sıfırdan sınama yazmak değil, elde olan koşuları
yinelenebilir hâle getirmek.**

---

## III · Yapılacaklar

### 1 · Her kanal için bir ayrıştırma sınaması — çevrimdışı fikstürle

Her kanalda en az bir sınama, **kaydedilmiş bir yanıt** üzerinden `parse`/`fetch`
yolunu koşturmalı ve alanları doğrulamalı (başlık, bağlantı, tarih, gövde).

- Fikstürler `crates/agent-reach-channels/tests/fikstur/<kanal>.json` (ya da `.html`)
  altında dursun. Kaynağı `verification_log.json`'daki gerçek çağrılar olsun —
  aynı URL'ler, bir kez çekilip diske alınmış hâlleri.
- **Sınamalar ağa çıkmasın.** Ağa çıkan sınama CI'da rastgele kırmızı yanar,
  sonra susturulur, sonra kimse bakmaz. Çevrimdışı olsun ki hep koşsun.
- Ağa gerçekten çıkan uçtan uca koşu ayrı kalsın: `#[ignore]` ile işaretlensin,
  elle `cargo test -- --ignored` ile koşulsun. CI'ya girmesin.

**Kapı — dördüncü kural:** her yeni sınama önce **kırmızı** gösterilecek. Ayrıştırıcıyı
kasten boz (bir alanı yanlış anahtardan oku), kırmızıyı rapora yapıştır, düzelt.
Kırmızı yanmayan sınama teslim edilmez.

### 2 · Düşemeyen sınamayı düzelt

`linkedin.rs`. İki seçenek, biri seçilip gerekçesi yazılsın:
- Kurulum durumu sınamada **sabitlensin** (sahte `Config` ile tek beklenen sonuç), ya da
- Sınama ikiye ayrılsın: kurulu değilken `NotInstalled`, kuruluyken `RequiresConfig`.

### 3 · Kapsama kapısı — bir sonraki turda kimse unutmasın

`tests/kapsama.rs` diye bir sınama: `channels/src/` altındaki her kanal dosyası için
karşılık gelen bir fikstür sınamasının **var olduğunu** doğrulasın. Yeni kanal
eklenip ayrıştırma sınaması yazılmazsa CI kırmızı yansın.

Bu, on birinci kuralın kod hâli: bakılmayan yol "yok" diye görünmesin.

### 4 · Kök dizin temizliği

`verification_log.json` ile `test_execution_log.json` **ikisi de 4.648 bayt** —
büyük ihtimalle aynı koşunun iki kopyası. `.gitignore` bu kalıpları içeriyor ama
dosyalar zaten commit'li olduğu için dışlama iş görmüyor.

Fikstürlere dönüştürüldükten sonra: bir tanesi `docs/olcum/` altına kanıt olarak
taşınsın, ötekiler `git rm --cached` ile takipten çıkarılsın.

### 5 · CI'ya iki satır

```yaml
- run: cargo test --workspace --verbose
- run: cargo test --workspace -- --ignored   # elle tetiklenen isde, agli kosu
```

İkincisi `workflow_dispatch` ile ayrı bir işte koşsun; her push'ta değil.

---

## IV · Sıra

1. Bir kanalla başla — **`github.rs`** (yanıtı kararlı, belgelenmiş, kimlik
   gerektirmeden okunabilir uçları var). Fikstür + ayrıştırma sınaması + kırmızı kanıtı.
   Bu, kalan 12'ye şablon olur.
2. `linkedin.rs`'in düşemeyen sınaması.
3. Kalan 11 kanal, şablonla.
4. `tests/kapsama.rs`.
5. Kök temizliği ve CI.

**Yeni kanal eklenmesin** — bu on üçünün davranışı sınanana kadar. On dördü
sınanmayan bir araç, on üçü sınanmayandan daha iyi değil.

---

## V · Bildirme — ZOPAY kurallarının burada da geçerli olanları

- **Onuncu kural:** her başlığın altında onu üreten commit'in kısa sha'sı.
- **On sekizinci kural:** `geçti / kaldı` **çifti** yazılır. "20 geçti" tek başına
  yazılmaz.
- **Sekizinci kural:** raporda "neyin sınanmadığı" ayrı bir başlık olsun.
- **"Tam başarı" ve "kalan iş yok" ifadeleri kullanılmaz.**
- Sayı yazılacaksa onu üreten komut ve çıktısı rapora girsin.

Rapor `docs/RAPOR-01-ELKASSAM.md` olarak yazılsın.

---

## VI · Dokunulmayacaklar

- **Bu depo ZOPAY'a katılmaz.** Ayrı araç, ayrı depo, ayrı sürüm.
- Çalışan 13 kanalın davranışı değiştirilmesin; bu tur **yalnız kapı** turu.
  Ayrıştırıcıda bir kusur bulunursa ayrı commit'te ve sınamasıyla düzeltilsin.
- Ağa çıkan sınama CI'ya konmaz.
