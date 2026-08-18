# Bilet C — öğrenici, tek eksenli

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
- Kip: **gölge.** Çıktı kullanıcıya gösterilmez, arka planda ölçülür.

## Kabul ölçütü

**Gauntlet'i Tur B'nin bıraktığı sayının üstüne çıkarır.**

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
