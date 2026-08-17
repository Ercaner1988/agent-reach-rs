# 🏛️ TEKNİK VE FELSEFİ UYGULAMA RAPORU: 4 BOYUTLU EPİSTEMİK SEMANTİK ZİHİN HARİTASI VE ÇİFT KANALLI ÖĞRENEN ARAMA MİMARİSİ

**Hazırlayan:** Kassam  
**Sunulan:** Ercan ER & Mihenk  
**Tarih:** 18 Ağustos 2026  
**Proje:** `agent-reach-rs` (`crates/agent-reach-graph`)  
**Kural ve İlke:** Nisâ 135 (Adalet ve Hakikat Merkezli Entelektüel Dürüstlük)

---

## 📋 ÖZET VE STRATEJİK MİMARİ VİZYON

Bu rapor, `agent-reach-rs` arama motorunu statik ve kod içerisine sert biçimde yazılmış sorgu gevşetme kurallarından (`if query.contains(...)`) kurtararak; kendi kendine öğrenebilen, çok boyutlu kavram uzayında semantik bağıntıları yönetebilen ve dış arama motorlarına (DuckDuckGo, Exa, Google vb.) muhtaç kalmadan doğrudan internet gezintisi (Search-Engine Free Traversal) yapabilen canlı bir **Semantik Zihin Haritası Katmanı** kazandırma mimarisini sunmaktadır.

Yapı, `crates/agent-reach-graph` adı altında bağımsız bir pure-Rust SQLite (`minisqlite` / `libsql`) motoru üzerine kurulacak; başlangıçta statik kuralların arkasında **gölge kipinde (shadow execution)** çalışarak arka planda kendisini eğitecek ve olgunlaştığında ana arama motoru olarak öne çıkacaktır.

---

## 🧬 1. SAF RUST SQLITE KATMANINDA ÇOK BOYUTLU EPİSTEMİK VERİ MODELİ

### 1.1 Veritabanı Şeması (`crates/agent-reach-graph/schema.sql`)

Dış C/CGO bağımlılığı olmayan pure-Rust SQLite tabanlı ilişkisel tablo yapısı:

```sql
-- 1. Kavram Düğümleri Tablosu (Nodes)
CREATE TABLE IF NOT EXISTS dugumler (
    dugum_id INTEGER PRIMARY KEY AUTOINCREMENT,
    terim TEXT NOT NULL,
    dil TEXT NOT NULL CHECK(dil IN ('TR', 'ENG', 'AR')),
    kategori TEXT NOT NULL DEFAULT 'kavram', -- 'teknoloji', 'kavram', 'takma_ad', 'fiil'
    olusturma_tarihi DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(terim, dil)
);

-- 2. Çok Boyutlu Semantik Bağıntılar Tablosu (Edges)
CREATE TABLE IF NOT EXISTS bagintilar (
    baginti_id INTEGER PRIMARY KEY AUTOINCREMENT,
    kaynak_id INTEGER NOT NULL,
    hedef_id INTEGER NOT NULL,
    baginti_turu TEXT NOT NULL, -- 'ESANLAMLIDIR', 'ZITANLAMLIDIR', 'UYARLAMASIDIR', 'ILGILIDIR'
    
    -- Anlamsal Yakınlık/Zıtlık Skalası (-1.0 ile +1.0 arası)
    x_anlamsal REAL NOT NULL DEFAULT 0.0 CHECK(x_anlamsal BETWEEN -1.0 AND 1.0),
    
    -- 4 Temel Epistemik Boyut Eksenleri (-1.0 ile +1.0 arası)
    y_ontolojik REAL NOT NULL DEFAULT 0.0,      -- Varlık / Yapılabilirlik (Kolay - Zor)
    z_estetik REAL NOT NULL DEFAULT 0.0,        -- Biçim / Zarafet (Güzel - Çirkin)
    w_epistemolojik REAL NOT NULL DEFAULT 0.0, -- Hakikat / Mantık (Doğru - Yanlış)
    v_ahlaki REAL NOT NULL DEFAULT 0.0,        -- Etik / Değer (İyi - Kötü)
    
    -- Dinamik Katsayılar
    agirlik REAL NOT NULL DEFAULT 0.5 CHECK(agirlik BETWEEN 0.0 AND 1.0),
    baglam_etiketi TEXT DEFAULT 'genel',
    kullanim_sayisi INTEGER DEFAULT 1,
    son_kullanim DATETIME DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY(kaynak_id) REFERENCES dugumler(dugum_id) ON DELETE CASCADE,
    FOREIGN KEY(hedef_id) REFERENCES dugumler(dugum_id) ON DELETE CASCADE,
    UNIQUE(kaynak_id, hedef_id, baginti_turu)
);

-- 3. Hızlı Metin İndeksi (FTS5)
CREATE VIRTUAL TABLE IF NOT EXISTS dugumler_fts USING fts5(
    terim,
    dil,
    content='dugumler',
    content_rowid='dugum_id'
);
```

### 1.2 Rust Veri Yapıları (`crates/agent-reach-graph/src/model.rs`)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Dil {
    TR,
    ENG,
    AR,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpistemicVector {
    /// x-Ekseni: Anlamsal Yakınlık/Zıtlık [-1.0, +1.0]
    pub x_anlamsal: f32,
    /// y-Ekseni: Ontolojik Bağlam (Kolay - Zor)
    pub y_ontolojik: f32,
    /// z-Ekseni: Estetik Bağlam (Güzel - Çirkin)
    pub z_estetik: f32,
    /// w-Ekseni: Epistemolojik Bağlam (Doğru - Yanlış)
    pub w_epistemolojik: f32,
    /// v-Ekseni: Ahlaki Bağlam (İyi - Kötü)
    pub v_ahlaki: f32,
    /// Gelecekte eklenebilecek genişletilebilir ek boyutlar
    pub ek_boyutlar: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticNode {
    pub id: i64,
    pub terim: String,
    pub dil: Dil,
    pub kategori: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticEdge {
    pub id: i64,
    pub kaynak_id: i64,
    pub hedef_id: i64,
    pub baginti_turu: String,
    pub vektor: EpistemicVector,
    pub agirlik: f32,
    pub kullanim_sayisi: u64,
}
```

---

## 🧮 2. ÇOK BOYUTLU EPİSTEMİK UZAY MATEMATİĞİ VE DİNAMİK SÖNÜMLENME

### 2.1 Çok Boyutlu Epistemik Bağlam Uayzı
Semantik dünyada iki kavram $x$-ekseninde doğrudan ilişkili görünmese bile ($x = 0$), diğer epistemik boyut eksenlerinde birleşebilir.

$$\vec{E} = \begin{bmatrix} x_{anlamsal} \\ y_{ontolojik} \\ z_{estetik} \\ w_{epistemolojik} \\ v_{ahlaki} \end{bmatrix}$$

İki kavram arasındaki **Ağırlıklı Epistemik Uzaklık ($D_E$)**:

$$D_E(A, B) = \sqrt{ \alpha(x_A - x_B)^2 + \beta(y_A - y_B)^2 + \gamma(z_A - z_B)^2 + \delta(w_A - w_B)^2 + \epsilon(v_A - v_B)^2 }$$

Burada $\alpha, \beta, \gamma, \delta, \epsilon$ bağlam ağırlık katsayılarıdır.

### 2.2 Dinamik Öğrenme ve Üstel Sönümlenme (Exponential Decay)

Arama motorunda bir kavram bağıntısı başarıyla eşleştiğinde bağlantı ağırlığı artırılır:

$$w_{yeni} = \min(1.0, \; w_{eski} + 0.05)$$

Zamanla kullanılmayan veya başarısız olan bağıntılar zaman ekseninde sönümlenir:

$$w(t) = w_0 \cdot e^{-\lambda \cdot \Delta t}$$

* $\Delta t$: Son kullanımdan bu yana geçen gün sayısı.
* $\lambda$: Sönümlenme sabiti (örn. $\lambda = 0.01$).
* Ağırlık $0.10$ eşiğinin altına düşen geçici bağıntılar çizgeden otomatik temizlenir (pruning).

---

## 🔄 3. ÇİFT KANALLI (DUAL-TRACK) VE GÖLGE KİPİ (SHADOW EXECUTION)

### 3.1 Mimari Akış Şeması

```
                       [Gelen Arama Sorgusu]
                                 │
                 ┌───────────────┴───────────────┐
                 ▼                               ▼
       [Sekme 1: Statik Arama]         [Sekme 2: Gölge Semantik Zihin Motoru]
       (Mevcut Stage Gevşetme)         (crates/agent-reach-graph)
                 │                               │
                 ▼                               ▼
       [Kullanıcıya Sunulan Çıktı]     [Sessiz Çıktı & Değerlendirme]
                 │                               │
                 └───────────────┬───────────────┘
                                 ▼
                    [Başarım Karşılaştırma Engine]
                     - Semantik motor başarımını ölç.
                     - Başarılıysa katsayıları güncelle (+0.05).
                     - Recall@10 >= %87.5 olunca Statik Katmanı Devreden Çıkar.
```

---

## 🌐 4. DIŞ ARAMA MOTORLARINDAN BAĞIMSIZ DOĞRUDAN İNTERNET GEZİNTİSİ (SEARCH-ENGINE FREE DIRECT TRAVERSAL)

### 4.1 Kavram Yönelimli Doğrudan Gezinti
Dış arama motorlarına (DuckDuckGo, Exa vb.) muhtaç kalmamak için:
1. Semantik çizgeden alınan kök kavramlar ve alan adı şablonları (`github.com/`, `crates.io/crates/`) birleştirilir.
2. `reqwest` + pure-Rust HTML ayrıştırıcı (`scraper` / `html5ever`) ile hedef adres doğrudan ziyaret edilir.
3. Sayfadaki semantik başlıklar (`<h1>`, `<h2>`, `<meta description>`) ve hiperlinkler ayıklanarak zihin haritasına yeni düğümler ve bağıntılar olarak eklenir.

---

## 🌍 5. ÇOK DİLLİ (TR / ENG / AR) EPİSTEMİK UZAY ÇAPRAZ KÖPRÜLEME

Türkçe (`TR`), İngilizce (`ENG`) ve Arapça (`AR`) için müstakil semantik düğümler tanımlanır. Diller arası hizalama `ESANLAMLIDIR` veya `UYARLAMASIDIR` kenarları üzerinden kurulur:

- `TR`: `"hızlı metin arama"`
- `ENG`: `"fast text search"`
- `AR`: `"بحث سريع في النصوص"`

Bu üç düğüm epistemik uzayda aynı vektör koordinatlarına bağlanarak, Türkçe sorgu girildiğinde İngilizce veya Arapça kaynakların da arama sonuçlarına girmesi sağlanır.

---

## ⚖️ AKADEMİK ADABI AŞAN / ZORLAYAN, TEMELSİZ İDDİALAR, GENEL BAĞLAMA OTURMAYAN KOPUK YA DA ÇELİŞKİLİ KISIMLAR, HAVADA KALAN İDDİALAR RAPORU

**Nisâ 135 Gereği Şeffaf Eleştirel Denetim:**

1. **Soğuk Başlangıç (Cold Start) Riski:** 
   * *Açıklama:* Semantik veritabanı boşken zihin haritasının ilk aramalarında hiçbir yanıt üretememesi veya saçmalaması kaçınılmazdır.
   * *Çözüm/Hafifletme:* Çift kanallı gölge çalışma kipi tam olarak bu riski bertaraf etmek için kurgulanmıştır. Sistem ilk aşamada tamamen sessiz kalacaktır.

2. **Epistemik Boyutların (Estetik, Ahlaki) Puanlama Öznelliği:**
   * *Açıklama:* Yazılım kütüphaneleri veya arama kavramları için "Estetik" ($z$) veya "Ahlaki" ($v$) puanlar vermek öznel kalabilir ve otomatik veri toplama ile hesaplanması zordur.
   * *Çözüm/Hafifletme:* Başlangıçta ontolojik ($y$) ve epistemolojik ($w$) boyutlar otomatize edilecek; estetik ve ahlaki boyutlar kullanıcı/uzman yönlendirmeli (HITL) bırakılacaktır.

3. **Pure-Rust SQLite Üzerinde Vektör Uzaklık İndeksleme Yükü:**
   * *Açıklama:* SQLite özgün bir vektör veritabanı değildir. Binlerce düğüm olduğunda Öklid/Kosinüs uzaklığı hesaplamak okuma performansını düşürebilir.
   * *Çözüm/Hafifletme:* Veritabanı açılışta bellek içi (`:memory:`) önbelleğe yüklenecek, FTS5 filtrelenmesinden geçen aday düğümler üzerinde çok boyutlu uzaklık hesaplanacaktır.

4. **Arama Motorsuz Gezintinin İnternet Engelleri (WAF / CAPTCHA):**
   * *Açıklama:* Dış arama motoru kullanmadan doğrudan web sitelerine istek atmak Cloudflare/WAF ve IP engel duvarlarına takılabilir.
   * *Çözüm/Hafifletme:* Gezinti adımları kullanıcı isteklerine göre sınırlandırılacak ve uygun HTTP header/user-agent rotasyonları kullanılacaktır.

---

*Bu rapor Mihenk ile yapılacak teknik ve felsefi değerlendirme için tam ve eksiksiz bir zemin oluşturmaktadır.*
