# Yol haritası — yeni kaynaklar ve kaynak anahtarı

**Tarih:** 20 Ağustos 2026 · **Yazan:** Mihenk (Claude Opus 5)
**Durum:** kaynak anahtarı **kuruldu**; kaynaklar sıraya alındı

---

## 1 · Kaynak anahtarı — bugün kuruldu

İstenen: *"bazı kaynakları istediğimizde kapatabilmek mümkün olsun (reddit gibi)"*.

`~/.agent-reach/config.yaml`:

```yaml
disabled_channels:
  - reddit
  - quora
```

Ya da tek koşu için, dosyaya dokunmadan:

```bash
AGENT_REACH_DISABLED_CHANNELS=reddit,quora
```

Ad karşılaştırması büyük/küçük harf ve boşluk duyarsız; önek eşleşmesi yok
(`red` yazmak `reddit`'i kapatmaz). Hem CLI hem MCP yolunda geçerli.

**Neden ayrı bir hata:** kapalı kanal, bozuk kanal değildir. Ölçüldü:

```
kapali : Channel 'reddit' is switched off in config (disabled_channels)
acik   : Backend 'reddit' is not available: praw ✗ not installed: … ; reddit-api ⚠ requires co…
```

İlki bir karardır, ikincisi kovalanacak bir arıza. Çağıran ajan ikisine farklı
davranmalı.

---

## 2 · On üç kaynak, üç kuşak

Sıralama zorluk değil **erişim biçimi**: kaynağın kendi açtığı kapı hangisi.

### Kuşak A — açık kapı, giriş yok, bugün yazılabilir

| Kaynak | Kapı | Ölçüm |
|---|---|---|
| **Wikipedia** | REST API (`/api/rest_v1/`) | `200` doğrulandı |
| **Stanford Encyclopedia** (plato) | Statik HTML, kararlı URL | `robots.txt` `200`; yol bazlı kısıtlara uyulacak |
| **İslam Ansiklopedisi** (TDV) | Açık HTML | Giriş yok |
| **Pew Research** | Açık rapor + veri seti indirmeleri | Giriş yok |
| **Substack** | **Yayın başına RSS** (`<yayin>.substack.com/feed`) | **Zaten çalışıyor** — mevcut `rss` kanalı, sıfır yeni kod |
| **DergiPark** | Açık erişim; API/OAI ucu **belirsiz** | İki tahminim `404` verdi; uç bulunmalı |

Substack özellikle önemli: yeni kanal gerekmiyor, bugün `rss_fetch` ile
okunabiliyor. Yol haritasına yazılacak tek şey, ilgilenilen yayınların listesi.

### Kuşak B — resmî API, anahtar gerekir

| Kaynak | Kapı |
|---|---|
| **Elsevier / ScienceDirect** | `api.elsevier.com` — resmî API, kurumsal anahtar. Kazıma değil, sözleşmeli erişim |
| **Google Scholar'ın işlevi** | Scholar'ın kendisi değil: **OpenAlex**, **Semantic Scholar**, **Crossref**, **CORE**. Hepsi ücretsiz API, aynı literatür çizgesi, atıf sayıları dahil |

Anahtarlara ben dokunmam; sen kendi makinende ortam değişkeni olarak
tanımlarsın, `Config` onu okur — `exa_api_key` ve `github_token` gibi.

### Kuşak C — duvar var, ve duvarı aşmak senin kendi kuralını çiğner

Bunları bugün ölçtüm, tahmin değil:

**Google Scholar** — `scholar.google.com/robots.txt`:

```
User-agent: *
Disallow: /scholar
```

Site, otomatik erişimi adıyla yasaklıyor. API yok. Ölçekli erişim CAPTCHA'ya
çarpar ve onu geçmek **senin koyduğun sınırın birinci maddesi**. Bu yüzden
Scholar bir kanal olarak yazılmayacak; yerine Kuşak B'deki üç açık API kullanılacak
— aynı işi yapıyorlar, üstelik yapılandırılmış veriyle.

**JSTOR** — `robots.txt` `/action`, `/api`, `/citation`, `/stable/full` yollarını
kapatıyor. Meşru yol var ve ayrıdır: **Constellate / Data for Research**, metin
madenciliği için kurumla anlaşmalı erişim.

**Quora** — `robots.txt` kısıtlı, kullanım şartları kazımayı açıkça yasaklıyor.
Ayrıca giriş duvarı var. Otomatik kanal yazılmayacak.

**Google Play Books** — satın alınan kitaplar DRM'li, okuma API'si yok. Kendi
kütüphanenden makine okunur çıkarma yolu bulunmuyor.

### Kuşak D — giriş var ama meşru: insan kapısı modeli

**YÖK Ulusal Tez Merkezi** — ulusal, kamusal bir arşiv; giriş istiyor. Doğru
model Kervan'ınki:

1. **İnsan bir kez giriş yapar**, görünür pencerede.
2. Profil kalıcıdır (çerez + localStorage), ajan o profili sürer.
3. **Doğrulama duvarı çıkarsa** (CAPTCHA, 2FA, e-posta kodu) ajan **çözmez**;
   pencereyi insanın önüne getirir ve bekler.
4. **Şifre saklanmaz.** Ne yapılandırmada, ne bellekte, ne kütükte.

Bu model Kuşak C'yi kurtarmaz — orada engel giriş değil, sitenin kendi
politikası — ama D için doğru ve yeterli.

---

## 3 · Camelira bir kaynak değil, bir araç

`camelira.abudhabi.nyu.edu` ayakta (`200`), ama yaptığı iş arama değil:
Arapça **morfolojik çözümleme ve belirsizlik giderme**. Yani ARR'ın "eriş ve
oku" kanallarından biri olamaz; sözcük normalleştirme aşamasında kullanılacak
bir **işlemci**. Tez tarafındaki Arapça metinler için değerli, ama kanal
listesine değil, metin hattına girer.

---

## 4 · Sıra

**Şimdi (kuşak A, düşük risk, ölçülebilir):**
1. Substack — kod yok, yalnız yayın listesi.
2. Wikipedia — REST API, tek kanal, kolay ölçülür.
3. Stanford Encyclopedia + İslam Ansiklopedisi — statik okuyucu, ortak bir
   "belge sitesi" kanalı olabilir.
4. Pew — rapor listesi + indirme.
5. DergiPark — önce ucu bul, sonra karar ver.

**Sonra (kuşak B):**
6. OpenAlex / Semantic Scholar / Crossref — Scholar'ın işlevi, açık kapıdan.
7. Elsevier — kurumsal anahtar geldiğinde.

**Sonra (kuşak D):**
8. YÖK Tez — insan kapısı modeliyle, görünür pencere, şifresiz.

**Yazılmayacaklar ve sebebi belgede:** Google Scholar, JSTOR, Quora, Play Books.

---

## 5 · Her yeni kanalın karşılaması gereken şart

Bu depodaki kural değişmiyor: **yeni bir kanal, altın kümede en az bir vaka
kazanmadan "çalışıyor" sayılmaz.** Bilet A'nın karşılanmamış şartı hâlâ açık —
altın kümenin 24 vakasının hepsi github hedefli. İlk açık erişimli kaynak
eklendiğinde o boşluk da kapanır, ve ölçüm tek kanal ailesine bakmaktan çıkar.
