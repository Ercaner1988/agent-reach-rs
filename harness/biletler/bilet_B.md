# Bilet B — merdiveni bitir

## Bağlam

`github.rs`'teki gevşetme merdiveni (`relaxation::ladder`) ölçülmüş hâliyle 16
vakanın 10'unu buluyor. Önceki hâli 1/16 idi; aradaki farkı yapan üç şey oldu:
altın kümeden kopyalanmış kelime listesi silindi, dil adı `--language`'e taşındı,
`--sort stars` eklendi.

Kalan altı kaçırma: **`#1, #5, #7, #8, #13, #16`.**

Bu bilet açılmadan önce Bilet A yeşil olmalı. Kısıtlanmış ölçümün üstünde merdiven
ayarlamak, gürültüye göre ayar yapmaktır.

## Görev

Altı kaçırmayı teşhis et ve merdiveni düzelt. Bakılacak yerler — sırayla:

- `--sort stars` hangi basamaklarda uygulanıyor? İlk basamakta yok (kasıtlı:
  birebir sorgu için alaka sıralaması doğru olabilir). Ölç, gerekiyorsa değiştir.
- `--limit 20` yeterli mi? `recall@10` ölçülüyor ama round-robin harmanlama
  basamaklar arasında sıra tüketiyor.
- Tek-terim basamağının seçimi: bugün `first` ve `longest` denenıyor. Bu altı
  vakada hangisi tutuyor, hangisi boşa gidiyor?
- İşlev kelimesi listesi eksik mi? **Yalnız dilbilgisel işlev kelimeleri eklenir.**
  Ölçüt tektir: kelime konu anlamı taşıyorsa listeye giremez.

## Kabul ölçütü

**github recall@10 ≥ 13/16 ölçülmüş, sıfır sonuç 0.**

Ölçülmüş demek: `Not measured` sıfır olan bir koşuda. Kısıtlanmış koşudan sayı
alınmaz.

## Demir kurallar

Bilet A'daki altı kural aynen geçerlidir. Bu bilete özel iki tanesi:

7. **Kelime listesi yasak.** Altın kümenin sorgularından türetilmiş hiçbir öbek
   koda girmez. Kapı 5 denetliyor; yorumlar da dahil.
8. **Ayırt edici kelime silinmez.** `headless`, `webdriver`, `api` gibi kelimeler
   sinyaldir; onları atmak sorguyu çözmez, sorguyu yok eder.

## Kapsam dışı

- Altın kümeye dokunmak (A'da kilitlendi)
- Yeni kanal, yeni bağımlılık, trait değişikliği
- Öğrenici/çizge katmanı — o Tur C

## Teslim

1. Altı kapı yeşil çıktısı.
2. Gauntlet koşusu: `Not measured` sıfır, github sayısı.
3. Kalan kaçırma varsa **teşhisiyle**: "merdiven #N'i kurtaramıyor çünkü X".
4. Ponytail notu: net satır değişimi `+X / -Y`.
