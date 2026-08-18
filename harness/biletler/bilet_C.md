# Bilet C — öğrenici, tek eksenli

## Durum — 19 Ağustos 2026: **HAK EDİLDİ**

Bilet C'nin 9. kuralı şunu istiyordu: *"Yeni bir kolon eklemenin şartı, onu
dolduran bir süreç ve onun işe yaradığını gösteren bir sayıdır."* O sayı artık
var, ve şöyle ölçüldü.

Üç sorgu iki motordan da boş dönüyor:

| # | Sorgu | Hedef | Hedefin açıklaması |
|---|---|---|---|
| 21 | `gorsel terminal arayuz kutuphanesi` | `ratatui/ratatui` | "cooking up **terminal user interfaces (TUIs)**" |
| 22 | `hızlı ve güvenilir ağ iletişim kütüphanesi` | `hyperium/hyper` | "An **HTTP** library for Rust" |
| 24 | `parse atom and news site updates in rust` | `rust-syndication/rss` | "serializing the **RSS** web content syndication format" |

Üçü de aynı şey: **derlemin kullandığı terimi sorgu hiç söylemiyor.** Sorgu
gevşetmeyle kapanmadıkları ölçüldü — hepsi denendi, hiçbiri ulaşmadı:

```
merdivenin dört basamağı (birebir · içerik+dil+yıldız · ilk terim · en uzun terim)
gh search --language rust --sort stars
in:name,description,readme      → awesome-list gürültüsü
in:name,description,topics      → hedefler büyük depolarca bastırılıyor
exa, num_results 10 ve 25       → kavramı anlıyor, hedefi ilk 25'e sokmuyor
```

Exa'nın #22'ye verdiği yanıt bunu iyi gösteriyor: Türkçe sorguya ağ
kütüphaneleri döndürdü, içinde `hyperium/h3` — hedefin **kardeş deposu**. Kavram
doğru anlaşılıyor, köprü kurulamıyor.

**Kestirme yol denendi ve düzenek reddetti.** `("terminal arayuz","tui")`,
`("news site","rss")` eşlemelerini koda koymak üçünü de kapatırdı; Kapı 5 ikisini
de adıyla yakaladı. Cevap anahtarından türetilen sözlük, sözlük değil kopyadır.

Yani öğrenicinin **ne yapması gerektiği** artık ölçülmüş bir soru: bu üç köprüyü,
cevap anahtarına bakmadan, gerçek arama sonuçlarından öğrenebiliyor mu.

**Soğuk başlangıç uyarısı:** bu üç sorgu hiç başarılı olmuyor, yani onlardan
öğrenilecek sinyal yok. Öğrenicinin tohumu başka yerden gelmeli — depo konu
etiketleri (`ratatui → tui, terminal-user-interface`; `hyper → http`;
`rss → feed, parser`) ölçüldü ve mevcut. Bu, cevap anahtarı değil, ortak veri.

---

## Bağlam

Bu bilet **yalnız Bilet A ve B yeşilse** açılır. Sebebi mimari değil, epistemik:
öğrenicinin yakıtı başarı/başarısızlık sinyalidir, ve geçen turda o sinyalin
15/16'sı hız sınırıydı. Bozuk ölçümün üstüne kurulan öğrenici bozukluğu öğrenir.

Kapatılacak gerçek boşluk şu: bazı sorgular her iki motorun da kaçırdığı bir
kavram sıçraması gerektiriyor — tarif edilen yetenek ile projenin adı arasında
hiçbir kelime örtüşmesi yok. Elle kural yazmak yerine bunu **öğrenmek** doğru.

## Görev

`crates/agent-reach-graph` — yeni sandık, **tek eksenli**.

- Depolama: `turso` `0.7.2` (saf Rust SQLite, MIT). `cursor/minisqlite`
  **kullanılamaz**: lisansı yok (`LICENSE` dosyası yok, `Cargo.toml`'da `license`
  alanı yok) ve `CREATE VIRTUAL TABLE` desteklemiyor. Bkz. ADR 0001.
- Şema: iki tablo. `dugumler(id, terim)` ve `bagintilar(kaynak, hedef, agirlik)`.
  **Beş boyut değil, bir ağırlık.**
- Öğrenme: arama başarılı olduğunda ilgili bağın ağırlığı artar; kullanılmayan
  bağ zamanla sönümlenir.
- Kip: **gölge.** Genişletilmiş sorgu kullanıcıya gösterilmez; arka planda
  çalışır ve sonucu ölçülür.
- **Ulaşılabilirlik şartı.** Gölge kip "ölü kod" demek değil: sandık kanaldan
  çağrılacak ve gauntlet onu görecek. Derlenip hiçbir yerden çağrılmayan sandık
  teslim sayılmaz — bu tam olarak geçen turun kusuruydu. Kapı 5'in bakacağı yer:
  `crates/*/src/` içinde `agent_reach_graph` geçiyor mu.

Not: `crates/agent-reach-graph` **silindi** (`191 satır, sıfır çağıran, beş
epistemik eksen — üçü bu biletin kapsam dışı listesinde`). Sıfırdan yazılacak.

## Kabul ölçütü

**Sıfır sonuç 3/24'ten 0'a iner** ve birleşik recall 21/24'ün üstüne çıkar.
(19 Ağustos ölçümü: github 16/24, exa 18/24, birleşik 21/24, ölçülemedi 0.)

Çıkarmıyorsa çizge büyütülmez. Sebebi yazılır ve bilet kapanır — bu bir
başarısızlık değil, doğru sonuçtur.

## Kapsam dışı — açıkça

Bunlar Tur C'nin işi **değil**. Her biri kendi ölçümüyle ayrı bilet olur:

- Dört epistemik eksen (ontolojik, estetik, epistemolojik, ahlaki). Bugün onlara
  hiçbir süreç yazmıyor; açılırlarsa kalıcı `0.0` kolonları olurlar. İkinci boyut,
  kendisini besleyen bir süreç ve onu doğrulayan bir ölçüm kazandığı gün açılır.
- TR/ENG/AR müstakil semantik uzaylar ve çapraz köprüleme.
- Arama motorsuz doğrudan gezinti. Bilinen adresten okumak zaten çalışıyor;
  arama motorunun yerine geçmek bot denetimini aşmayı gerektirir ve o yasaktır.

## Demir kurallar

Bilet A'daki altı kural aynen geçerli. Ek olarak:

9. **Ölçüm boyutu hak etmeli, tasarım değil.** Yeni bir kolon eklemenin şartı,
   onu dolduran bir süreç ve onun işe yaradığını gösteren bir sayıdır.
