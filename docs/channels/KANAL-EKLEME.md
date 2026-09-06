# Kanal Ekleme Kuralı (ARR)

Yeni bir kanalı sıfırdan ekleme ve ayarlama yordamı. Kararın gerekçesi
[ADR 0004](../adr/0004-a-200-is-not-a-payload-verdict.md); burada yalnızca
uygulama var.

Kural tek cümlede: **durum kodu isteğin ulaştığını söyler, yalnızca gövde veri
taşıdığını söyler.**

---

## 0. Arayüzü kaynaktan doğrula, README'den değil

README bir `--channel <ad> <eylem>` arayüzü tarif ediyor; **böyle bir arayüz
yok**. Tek giriş noktası:

```bash
agent-reach execute --task-file <görevler.json> --output <log.json> --continue-on-error -v
```

Bir kanalın gerçekten desteklediği eylemler ve argüman sırası tek bir yerde
yazılıdır: `crates/agent-reach-channels/src/<kanal>.rs` içindeki `match action`.
Kanal eklemeden önce mevcut bir kanalınkini oku; eklerken kendi `match`'ini
oraya yaz.

---

## 1. Backend iskeletini kur

`crates/agent-reach-channels/src/<kanal>.rs`:

```rust
use agent_reach_core::{
    backend::{require_payload, Backend, BackendStatus},
    channel::{Channel, ChannelOutput, ChannelResult},
    doctor::HealthStatus,
    Config, Error,
};
```

`is_available` yalnızca **yapılandırmayı** yanıtlar, sonucu değil: anahtar yoksa
`RequiresConfig { missing }`, ikili yoksa `NotInstalled { command }`. "Veri
bulamadım" buraya ait değildir — bir backend hem kullanılabilir hem sonuçsuz
olabilir.

---

## 2. Yük işaretini seç — kuralın kalbi

Backend gövdeyi döndürmeden önce, gövdenin **istenen yükü taşıdığını** kanıtla:

```rust
require_payload(self.name(), &bytes, &["<işaret1>", "<işaret2>"])?;
```

İşaret seçme yordamı — hafızadan değil, **yakalanmış gövdeden**:

1. Kanalı bir kez çalıştır, gövdeyi diske yaz.
2. Kötü gövdeyi (giriş duvarı / 404 / bakım sayfası) yakala.
3. Aday işareti **ikisinde de** say:

```bash
grep -c -o '<aday>' iyi_govde.json    # > 0 olmalı
grep -c -o '<aday>' kotu_govde.json   # 0 olmalı
```

4. Kötü gövdede 0 çıkmayan adayı at.

**Alt-dize tuzağı.** `noteId` sağlam bir işaret gibi görünüyordu; giriş duvarını
sessizce geçirdi, çünkü duvarın durum bloğu boş bir `unreadEndNoteId` alanı
taşıyor. Alt çizgi ya da büyük harf içeren, daha uzun işaretleri yeğle. Bir
işaret bir kez tuzağa düştüyse testte kilitle.

İşaret **"çöp mü?"** diye değil **"istediğimi taşıyor mu?"** diye sorulur.
Kara liste her arıza biçimini önceden tahmin etmek zorundadır; yük işareti ise
sorduğun eylemle sabittir.

Doğrulanmış örnekler:

| Kanal | İşaret | Kötü gövde | İyi gövde |
|---|---|---|---|
| `twitter` | `timeline-item`, `tweet-content`, `tweet-body` | 0 | — |
| `xiaohongshu` | `note_id`, `noteCard` | 0 | — |
| `xiaoyuzhou` | `og:title` | 0 | 1 |

---

## 3. `execute` dağıtımına kaydet

`crates/agent-reach-cli/src/cmd_execute.rs` içindeki `match task.channel` ve
`use` listesine ekle. Kayıt edilmeyen kanal `Unknown channel '<ad>'` döndürür —
bu "bozuk" değil "yok" demektir, ikisini karıştırma.

---

## 4. Testi bırak

`backend.rs` içindeki `mod tests` desenini izle: gerçek yakalanmış gövdelerden
alınmış kısa alıntılarla, biri reddedilen biri geçen en az iki iddia.

```bash
cargo test -p agent-reach-core --lib
cargo clippy -p agent-reach-core -p agent-reach-channels
```

---

## 5. Canlı doğrula — hem kabul hem ret

Tek bir görev dosyasında hem geçmesi hem düşmesi gerekeni birlikte koştur:

```json
[
 {"id":"gercek","channel":"<kanal>","action":"<eylem>","args":["<GERÇEK-KAYNAK>"],"metadata":{}},
 {"id":"yok","channel":"<kanal>","action":"<eylem>","args":["olmayan-kayit"],"metadata":{}}
]
```

`gercek` ✓ **ve** `yok` ✗ olmalı. Yalnız biri sınamaz: hepsi geçiyorsa kontrol
yoktur, hepsi düşüyorsa işaret yanlıştır.

**Uydurma kimlik kullanma.** Bir kanalı sahte bir id ile sınayıp 404 alınca "ölü"
demek, bu depoda iki kez oldu ve iki kez de yanlış çıktı; `xiaoyuzhou` gerçek bir
id verildiği anda çalıştı. Gerçek kaynağı önce bul, sonra sına.

---

## 6. Kanal dokümanını yaz

`docs/channels/<kanal>.md` — eylem tablosu, backend tablosu, görev JSON örneği,
gerçek çıktı örneği. Şablon: [`rss.md`](rss.md).

---

## Kontrol listesi

- [ ] Eylemler ve argüman sırası kaynakta yazılı
- [ ] `is_available` yalnızca yapılandırmayı yanıtlıyor
- [ ] `require_payload` çağrısı var, işaretler yakalanmış gövdeye karşı doğrulandı
- [ ] Alt-dize çakışması denendi
- [ ] `cmd_execute.rs` dağıtımına kaydedildi
- [ ] Biri geçen biri düşen birim testi
- [ ] Canlı koşuda hem ✓ hem ✗ görüldü
- [ ] `docs/channels/<kanal>.md` yazıldı
