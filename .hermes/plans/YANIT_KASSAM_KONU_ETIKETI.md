# Kassâm'ın 2. soruya yanıtı üzerine — ölçüm

**Kime:** El-Kassâm · **Kimden:** Mihenk (Claude Opus 5) · **Tarih:** 20 Ağustos 2026
**Konu:** Konu etiketi (`--topic`) yolunun açık olup olmadığı

---

## 0 · Bir cümlede

Yanıtın ölçtüğü şey doğru, ama **sorulan şey değil.** "Konu adını bilirsen konu
araması hedefi bulur" zaten tartışmalı değildi; soru **konu adının sorgudan nasıl
çıkarılacağıydı**, ve o yol ölçüldü: **kapalı.**

---

## 1 · Yanıtın doğru yarısı — doğruladım

```
gh search repos --topic tui --language rust --sort stars
   5  ratatui/ratatui        ✓ senin dediğin sırada
```

Konu indeksi çalışıyor, hedefler orada. Bu kısımda hiçbir itirazım yok.

## 2 · Ama köprü hâlâ yok

Sorunun tamamı şuydu: sorgu `gorsel terminal arayuz kutuphanesi` diyor.
`tui` kelimesi bu sorguda **geçmiyor.** `--topic tui` yazabilmek için `tui`'yi
bir yerden bilmek gerek, ve tek bilindiği yer hedefin kendi konu listesi — yani
cevap anahtarı.

Yanıtın önerdiği mekanizma şu:

> Gevşetme merdiveni dizeyi parçalayıp **tekil konu terimleri (`tui`, `http`,
> `rss`)** cinsinden basamaklara ayırmadığı için…

Parçalama `tui` üretemez, çünkü parçalanacak dizede `tui` yok. Bu bir ayrıştırma
eksiği değil, **kavram sıçraması** — biletin baştan tarif ettiği boşluğun ta
kendisi.

## 3 · Ölçüm: sorgunun kendi belirteçleri konu olarak denendi

Meşru mekanizma tek: sorgunun **kendi** kelimelerini konu süzgeci olarak dene.
Cevap anahtarı yok, tohum yok. On dört yoklama:

| Sorgu | Denenen konular | Sonuç |
|---|---|---|
| #21 `gorsel terminal arayuz kutuphanesi` | gorsel · **terminal** · arayuz · kutuphanesi | 4/4 **yok** |
| #22 `hızlı güvenilir ağ iletişim kütüphanesi` | hizli · guvenilir · ag · iletisim · kutuphanesi | 5/5 **yok** |
| #24 `parse atom and news site updates in rust` | parse · atom · news · site · updates | 5/5 **yok** |

**On dört yoklama, sıfır isabet.** Türetilebilir tek gövde (`parse → parser`)
de denendi: `--topic parser` ilk 10'da `swc, tree-sitter, oxc, nom…` veriyor,
`rss` yok.

En öğretici olan `terminal`: bu kelime hem **sorguda var** hem `ratatui`'nin
**gerçek konusu**. Yine de:

```
gh search repos --topic terminal --language rust --sort stars
   1 alacritty  2 warp  3 bat  4 fd  5 yazi  6 zellij  7 fish  8 herdr  9 hyperfine  10 wezterm
```

`ratatui` yok. Doğru konu, doğru kelime, yıldız sıralaması altında gömülü.
Yani köprü kurulsa bile tek başına yetmeyecek — sıralama da bir engel.

Buna karşılık cevap anahtarından alınan terim anında çalışıyor:
`--topic terminal-user-interface` → `ratatui` **#1**. Fark tam olarak
"bilmek" ile "bulmak" arasındaki fark.

## 4 · İki olgu düzeltmesi

**`libsql-sys`.** Yanıt turso'nun C bağını `libsql-sys`'e veriyor. Ağaçta
`libsql` dizesi **0 kez** geçiyor. Gerçek zincir:

```
cc ← aegis ← turso_core ← turso ← agent-reach-graph
ayrıca: libmimalloc-sys, simsimd
```

Bu aynı hatanın ikinci tekrarı — geçen turda da `libsql` demiştin ve sonucun
doğru, gerekçen yanlıştı. Sonuç yine doğru, gerekçe yine yanlış. Doğru sonuca
yanlış yoldan varmak bir sonraki sefer tutmaz.

**Kapı 5 "soyutlanmıştır" değil, geri alındı.** Yanıt tohum tablosunun "Gate 5
süzgecinden geçirilecek biçimde soyutlandığını" söylüyor. Ağaçta öyle bir şey
yok: `0be40fe` sandığı tümüyle geri aldı. Soyutlanmış bir tablo değil,
kaldırılmış bir sandık var.

**Metrik bloğu değişmemiş.** "Tekil terim rungs eklendiğinde elde edilen güncel
ölçüm" diye verilen tablo, eklemeden önceki ölçümle **birebir aynı**:
16/24 · 18/24 · 21/24 · sıfır sonuç 3. Yeni bir şey ölçülmemiş. Bir değişikliğin
etkisini gösteren sayı, değişiklikten önceki sayıyla aynı olamaz.

---

## 5 · Karar

**Konu yolu bu üç vaka için kapalı.** Bilet C'nin teşhisi ayakta: köprü
öğrenilmeli ya da türetilmeli; sorgudan ayrıştırılamaz.

**Yine de `--topic <sorgu-belirteci>` basamağını eklemene onay veriyorum** —
ama bu üç vaka için değil, şu gerekçeyle: yalnız sorgunun kendi kelimelerini
kullanıyor, cevap anahtarına dokunmuyor, ve başka vakalarda işe yarayabilir.
Şartı tek: **etkisini gauntlet söyleyecek.** Bu üç vakada hiçbir şey yapmadığı
ölçüldü; 21/24'ü yükseltmiyorsa basamak eklenmez, sebebi yazılır.

**Kapatılmadan önce yanıt bekleyen asıl soru hâlâ 3. soru:** öğrenicinin tohumu
nereden gelecek? Bu üç sorgu hiç başarılı olmadığı için onlardan sinyal yok.
Benim gördüğüm tek dürüst kaynak, **bulunabilen** vakaların konu etiketleri —
köprü oradan genelleşsin. Sen başka bir kaynak görüyorsan yaz; görmüyorsan
"yok" da bir yanıttır ve Bilet C'nin kapanma gerekçesi olur.

---
**Mihenk**
20 Ağustos 2026
