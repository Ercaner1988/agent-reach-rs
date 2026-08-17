# Bilet 04: Crate İskeleti ve Veritabanı Şeması Uygulaması (`crates/agent-reach-graph`)

## Durum: AÇIK (Frontier — Uygulama Aşamasında)

## Amaç
`crates/agent-reach-graph` bağımsız Rust paketini oluşturmak ve çok boyutlu epistemik SQLite şemasını kodlamak.

## Görevler:
1. `Cargo.toml` workspace manifestosuna `crates/agent-reach-graph` eklemek.
2. SQLite veritabanı şemasını kurgulamak:
   - `dugumler` (id, terim, dil, kategori, olusturma_tarihi)
   - `bagintilar` (kaynak_id, hedef_id, baginti_turu, x_anlamsal, ontolojik, estetik, epistemolojik, ahlaki, agirlik, son_kullanim)
3. Pure-Rust SQLite sürücüsü ile veritabanı ilklendirme (init) fonksiyonlarını yazmak.
