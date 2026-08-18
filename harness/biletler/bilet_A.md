# Bilet A — ölçümü dürüst ve dolu hâle getir

## Bağlam

`agent-reach-rs` arama katmanının ölçüm düzeneği var ama iki kusuru taşıyor.

**Birincisi: küme yetersiz.** 16 vakanın son sekizi (`#9–16`) hedefin adını sorgunun
içinde taşıyor — `ripgrep`, `fd`, `bat`, `dust`, `uv`, `polars`, `Deno`. "Adını
bildiğim şeyi bul" bir arama sınaması değil, arama-kutusu sınamasıdır. Gerçek zorluğu
ölçen ilk sekiz vaka gibi, hedefi tarif eden sorgulara ihtiyaç var.

**İkincisi: ölçüm kısıtlanıyor.** `exa` HTTP 429, `duckduckgo` HTTP 202 veriyor.
Bu proje parmak izi sahteciliği ve vekil döndürme yapmadığı için hız sınırları
aşılmaz — ölçüm kıt bir kaynaktır ve bütçelenmelidir. Geçen tur bu yüzden çöktü:
16 sorgu aralıksız koşuldu, uç 429 verdi, düzenek bunu "bulamadı" diye puanladı ve
0/16 bir yetenek sonucu sanıldı.

`Outcome::Unmeasured` ve kaset altyapısı zaten kurulu. Bu bilet onları doldurur.

## Görev

### A1 — Altın kümeyi 16'dan 24'e çıkar, ön-kayıtla

`crates/agent-reach-channels/tests/golden_search.json`. Mevcut on altı vakayı
**silme, değiştirme, yeniden sıralama.** Sekiz vaka ekle.

Her yeni vaka için, bu sırayla:

1. Sorguyu **önce** yaz — doğal cümle, sonucuna bakmadan.
2. Hedefi seç ve `gh repo view <hedef> --json nameWithOwner,stargazerCount` ile var
   olduğunu doğrula. En az 500 yıldız.
3. **Hedefin adı sorguda geçmeyecek.** `polars dataframe` gibi bir sorgu bu kurala
   takılır; `columnar dataframe library with lazy evaluation` takılmaz.

Küme şu üçünü de karşılamalı:
- En az iki vaka Türkçe ya da çeviriyazı farkı sınasın (`ı`/`i`, `ş`/`s`, aksan).
- En az bir vaka olumsuzlama içersin.
- En az bir vaka `github` dışı bir kanalın işi olsun (`rss`, `web`, `youtube`).

Bu dosyayı **kaynak koda hiç dokunmadan, tek başına** commit et:
```
test: lock the expanded golden set before implementation
```
Sıra bağlayıcıdır. Küme sonuçlara bakılarak yazılırsa ölçüm anlamını yitirir.

`search_gauntlet.rs`'teki `assert_eq!(test_cases.len(), 16, ...)` satırı 24 olur —
bu, hakem dosyasında bir eşik değişikliğidir, **ayrı commit'te** ve gerekçesiyle.

### A2 — Hız düzenlemesini gerçek uçlara göre ayarla

Bugün gauntlet sorgular arası 1500 ms bekliyor ve bu yetmiyor. Ölç ve ayarla:
uç başına en az 3 sn, 429/202 görüldüğünde üstel geri çekilme. Yeni bir mekanizma
kurma — `Outcome::Unmeasured` ve `is_throttle` zaten var.

### A3 — Kaseti doldur

`AGENT_REACH_CASSETTE` ayarlıyken tam bir gauntlet koş. Her sorgu bir kez ağa
çıkıp `harness/kaset/` altına yazılsın. Sonra aynı koşuyu tekrarla: ikincisi ağa
çıkmamalı ve belirgin biçimde hızlı bitmeli.

## Kabul ölçütü

**Arka arkaya iki tam koşuda `Not measured (throttled): ... combined 0`.**

Bu sağlanmadan hiçbir recall sayısı tartışılmaz. Sayının kendisi bu biletin konusu
değil — Tur B'nin konusu.

## Demir kurallar

1. **Derlenmeyen kod commit edilmez.** `pwsh -File harness/kapilar.ps1` altı kapıyı
   da koşar, bedava ve ~30 saniye. Altısı yeşil olmadan teslim etme.
2. **Altın kümedeki hiçbir dize kaynak dosyalarda geçmez** — yorumlar dahil.
   Kapı 5 bunu makineyle denetliyor.
3. **Eşik yalnız ayrı bir commit'te**, gerekçesiyle değişir; o commit başka bir şey
   içermez.
4. **Taşıma hatası (429/202/zaman aşımı) asla "bulamadı" diye puanlanmaz.**
5. Depo dili **İngilizce**: tanımlayıcılar ve belge yorumları. (Kervan Türkçedir,
   bu depo değil.)
6. **Uydurma yok.** Bir vaka geçmiyorsa geçmiş gibi raporlama.

## Kapsam dışı — biri gerekli görünüyorsa yapma, sor

- Yeni kanal eklemek
- `Channel` ya da `Backend` trait'ini değiştirmek
- Yeni bağımlılık (özellikle veritabanı — o Tur C'nin işi)
- Merdiveni (`relaxation::ladder`) değiştirmek — o Tur B'nin işi
- Canlı gauntlet'i sen koşmak — sürücü koşar

## Teslim

1. `harness/kapilar.ps1` tam çıktısı, altı kapı yeşil.
2. İki gauntlet koşusunun `Not measured` satırları yan yana.
3. Eklenen sekiz vakanın listesi ve her birinin `gh repo view` doğrulaması.
4. Hâlâ kısıtlanan uç varsa teşhisiyle — kaçırılanı bildirmek kusur değil,
   gizlemek kusurdur.
