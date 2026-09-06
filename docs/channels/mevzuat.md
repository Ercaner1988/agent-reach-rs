# Mevzuat Kanalı

Türk mevzuatı okuyucu kanal — [mevzuat.gov.tr](https://www.mevzuat.gov.tr/)
resmî portalı. Anahtar, hesap ya da çerez gerektirmez.

## Eylemler

| Eylem | Açıklama | Argümanlar |
|-------|----------|-----------|
| `mevzuat` | Kayıt sayfası: yayım bilgileri ve resmî `.doc`/`.pdf` bağlantıları | `[no]`, `[no, tur]` ya da `[no, tur, tertip]` |
| `metin` | Tam metin (maddeler) | aynı |
| `fihrist` | `metin` ile aynı uç | aynı |

Bir kayıt üç sayıyla adreslenir. Yalnız ilki gerçekten değişir: `MevzuatTur=1`
(*Kanun*) ve `MevzuatTertip=5` varsayılandır, dolayısıyla `["5237"]` tam bir
sorgudur. Diğer türler için ikinci ve üçüncü argümanı ver (örn. Cumhurbaşkanlığı
kararnameleri `tur=19`).

## Arka-uçlar

| Arka-uç | Açıklama | Kullanılabilirlik |
|---------|----------|------------------|
| `mevzuat-web` | Genel portal, HTTP | Her zaman |

## Yük imzası

Olmayan bir `MevzuatNo` **404 döndürmez**. `302` ile
`/Anasayfa/ErrorPage?code=404` sayfasına yönlendirir; reqwest bunu takip eder ve
arka-uca `HTTP 200` + ~65 KB hata sayfası ulaşır. Durum koduyla gerçek bir
kayıttan ayırt edilemez — [ADR 0004](../adr/0004-a-200-is-not-a-payload-verdict.md)
tam olarak bunu engeller.

İmzalar yakalanmış gövdelere karşı seçildi (TCK 5237 = iyi, 9999999 = kötü):

| Eylem | İşaret | İyi gövde | Kötü gövde |
|-------|--------|-----------|------------|
| `mevzuat` | `MevzuatMetin`, `MevzuatNo` | 2, 1 | 0, 0 |
| `metin` | `section-to-print`, `Madde` | 2, 352 | 0, 0 |

## Kullanım

### Görev JSON

```json
[
  {
    "id": "tck-metin",
    "channel": "mevzuat",
    "action": "metin",
    "args": ["5237"]
  },
  {
    "id": "anayasa-metin",
    "channel": "mevzuat",
    "action": "metin",
    "args": ["2709"]
  }
]
```

### Komut

```bash
agent-reach execute --task-file mevzuat_tasks.json --output mevzuat_log.json --verbose
```

### Çıktı

`metin` ham HTML döndürür; `output.text` alanında maddeler yer alır.

```
✓ tck-kayit via mevzuat-web (960ms)
✓ tck-metin via mevzuat-web (917ms)
✓ anayasa-metin via mevzuat-web (628ms)
```

TCK tam metninden (677 KB):

> Madde 1- (1) Ceza Kanununun amacı; kişi hak ve özgürlüklerini, kamu düzen ve
> güvenliğini, hukuk devletini, kamu sağlığını ve çevreyi, toplum barışını
> korumak, suç işlenmesini önlemektir.

Anayasa tam metninden (649 KB):

> Madde 1 – Türkiye Devleti bir Cumhuriyettir.

`mevzuat` eylemi resmî dosya bağlantılarını verir:
`MevzuatMetin/1.5.5237.doc`, `MevzuatMetin/1.5.5237.pdf`.

### Reddedilen kayıt

```
✗ olmayan: Backend 'mevzuat-web' execution failed: HTTP 200 but body carries
  no payload (none of section-to-print, Madde present in 64854 bytes)
```

## Bilinen sınırlar

- Arama yok. Kanal `MevzuatNo` ile adresler; numarayı bilmiyorsan önce
  `duckduckgo` ya da `web` kanalıyla bul.
- `mevzuat` eylemi kabuk sayfayı döndürür, madde metnini değil. Metin için
  `metin` kullan.
- Çıktı ham HTML; madde ayrıştırma çağırana bırakılmıştır.
