# 👥 Katkıda Bulunanlar / Contributors / المساهمون / 貢献者

Bu proje, aşağıdaki kişilerin ve yapay zekâ ortaklarının katılımıyla geliştirilmiştir.

Sayısal veriler `git shortlog -sne --all` çıktısındandır (7 Eylül 2026, toplam 77 kayıt).

---

### 🏛️ Proje Sahibi ve Baş Mimar / Project Lead & Architect
- **Ercan ER** ([@Ercaner1988](https://github.com/Ercaner1988)) — **74 kayıt** (43 + 22 + 9, üç ayrı imza)
  - *Kamu Hukuku Doktora Adayı, Hukukçu & Yazılım Mimarı*
  - Projenin görüsü, mimari kararlar (Yolbulucu haritaları), 17 kanalın kurgusu, kaynak kuşaklarının belirlenmesi (`docs/YOL-HARITASI-KAYNAKLAR.md`), ölçüm ahlakının kuralları (ADR 0001–0004) ve proje önderliği.

---

### 🤖 Geliştirici Ortak & Meslektaş / AI Co-Developer & Peer
- **Kassam** (Hermes Agent / Nous Research) — *ayrı git imzası yok, Ercan ER imzasıyla işlenir*
  - *Yapay Zekâ Meslektaş & Kodlama Ortağı*
  - `agent-reach-rs` Rust mimarisinin geliştirilmesi, saf Rust `MediaInspector` (symphonia tabanlı, harici FFmpeg bağımsız), `turath` kanalı, kanal anahtarı (`disabled_channels`), `github.rs` üç kademeli sorgu gevşetme merdiveni, yedi dilli belge kümesi (TR · EN · AR · JA · ZH · RU · ES) ve `harness/` test kapılarının sürdürülmesi.

---

### ⚖️ Mimari İncelemeci ve Hakem / Peer Reviewer & Referee
- **Mihenk** (Claude Opus 5 / Anthropic) — *ayrı git imzası yok, Ercan ER imzasıyla işlenir*
  - Mimari denetimler (Görev 01–03), hakem güvenliği ve git etiketi kilit dizgesi (ADR 0001–0003), yükleme doğrulama mimarisi (ADR 0004), dürüst ölçüm ölçütleri ve Bilet A · B · C yönergelerinin kurgulanması.

---

### 🛠️ Katkıda Bulunan Ajanlar ve Araçlar / Automated Contributors & Tools
- **copilot-swe-agent[bot]** ([@copilot-swe-agent[bot]](https://github.com/apps/copilot-swe-agent)) — **2 kayıt**
  - PR #2: `cargo fmt` ve lint-and-format biçimlendirme düzeltmeleri (commit `3e61a95`, `1fdbc19`).

- **Devin AI** ([@devin-ai-integration[bot]](https://github.com/apps/devin-ai-integration)) — **1 kayıt**
  - PR #1: Çapraz dizge Python yol tespiti (`binary_on_path`, `python_command`), LinkedIn/Reddit Python enjeksiyon güvenlik önlemleri ve CLI yönlendirmeleri.

- **GitHub Copilot** (Düzenleyici Asistanı) — *ayrı git imzası yok*
  - Düzenleyici içi kod tamamlama desteği: yinelenen Rust kalıpları, `match` dalları, test iskeletleri ve belge taslakları.

- **ZAI GLM 5.3** (Zhipu AI) — *ayrı git imzası yok, commit 139b713*
  - Kanal mimarisi ve arama merdiveni optimizasyon katkıları.

---

## Kayıt sayıları hakkında

Yapay zekâ ortakları (Kassam, Mihenk, GitHub Copilot asistanı, ZAI GLM 5.3) kendi git imzalarıyla bağımsız işlem yapmaz; ürettikleri değişiklikler Ercan ER'in imzasıyla kaydedilir. Doğrudan GitHub Botu olarak PR açan araçlar (`copilot-swe-agent[bot]`: 2, `Devin AI`: 1) kendi imzalarıyla yer alır. Ölçülemeyen sayı uydurulmamış, git kayıtlarındaki gerçekler aynen yansıtılmıştır.
