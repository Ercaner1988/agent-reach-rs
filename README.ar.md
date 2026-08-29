**🌍 [Türkçe](README.md) | [English](README.en.md) | [العربية](README.ar.md) | [日本語](README.ja.md) | [中文](README.zh.md) | [Русский](README.ru.md) | [Español](README.es.md)**

# Agent Reach RS (`agent-reach-rs`)

> **محرك قراءة البيانات والوسائط متعدد القنوات بلغة رستم الخالصة لوكلاء الذكاء الاصطناعي**

مشروع `agent-reach-rs` هو بيئة برمجية متكاملة بلغة Rust تمكّن وكلاء الذكاء الاصطناعي (Hermes, Claude, Codex, OpenCode) من قراءة البيانات بدقة وسرعة واستقلالية عبر المواقع الإلكترونية والشبكات الاجتماعية والمصادر الأكاديمية والوسائط متعددة الأوساط.

---

## 🎯 1. الأهداف والميزات

- **الاستقلالية التامة عن FFmpeg الخارجي (`MediaInspector`):** فك ترميز وفحص ملفات الصوت والوسائط (MP3, WAV, AAC, FLAC, OGG, MKV) بلغة Rust الخالصة عبر مكتبة `symphonia` (v0.5) دون الحاجة لملف `ffmpeg.exe` خارجي.
- **14 قراءة قنوات متعددة:**
  - **الشبكات والمواقع:** Twitter/X (Nitter / GraphQL), Reddit API, Bilibili, Xiaohongshu (XHS), V2EX, Xueqiu, LinkedIn, Xiaoyuzhou.
  - **الأكاديمي والكود:** قاعدة بيانات تراث (الفقه الإسلامي والمخطوطات), GitHub REST API, خلاصات RSS/Atom.
  - **محركات البحث:** البحث الدلالي Exa AI, محرك DuckDuckGo, محرك Jina Web Reader.
- **محرك المتجهات الإبستمولوجية خماسي الأبعاد (`agent-reach-graph`):** مصفوفة أبعاد أنطولوجية، جمالية، إبستمولوجية، أخلاقية ولغوية قائمة على Turso SQLite (0.7.2).
- **تكامل خادم MCP:** برنامج تشغيل خادم JSON-RPC متوافق تماماً مع معايير Model Context Protocol (MCP).

---

## 🏗️ 2. البنية المفهومية والوحدات

```text
agent-reach-rs/
├── Cargo.toml                    # إعدادات مساحة العمل (symphonia, tokio, reqwest)
├── crates/
│   ├── agent-reach-core/        # الأنواع الأساسية، MediaInspector، إدارة الأخطاء، الإعدادات
│   ├── agent-reach-channels/    # تنفيذ 14 قناة قراءة (YouTube, تراث, RSS الخ)
│   ├── agent-reach-mcp/         # مشغل خادم MCP JSON-RPC
│   └── agent-reach-cli/         # واجهة سطر الأوامر (الملف التنفيذي: agent-reach)
└── harness/                     # اختبارات التحقق التلقائي وبوابات التحكيم
```

---

## 🚀 3. التثبيت والإعداد

### المتطلبات الأساسية
- **أدوات Rust:** Rust 1.75+ (تثبيت `cargo` و `rustc`).
- **المتطلبات الخارجية:** لا يوجد (لا يتطلب FFmpeg خارجي أو Python أو Node.js).

### التجميع
```bash
# استنساخ المستودع
git clone https://github.com/Ercaner1988/agent-reach-rs.git
cd agent-reach-rs

# تجميع مساحة العمل
cargo build --release
```

سيكون الملف التنفيذي المجمع في المسار `target/release/agent-reach.exe`.

---

## 📖 4. الاستخدام والأمثلة

### أ. تحليل الوسائط بلغة Rust الخالصة (`MediaInspector` API)
```rust
use agent_reach_core::MediaInspector;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // تحليل الملف الصوتي مباشرة دون استخدام ffmpeg.exe
    let meta = MediaInspector::inspect_file("audio_sample.mp3")?;
    
    println!("الترميز: {}", meta.codec_name);
    println!("معدل العينة: {} Hz", meta.sample_rate);
    println!("عدد القنوات: {}", meta.channels);
    println!("المدة: {:.2} ثانية", meta.duration_seconds);
    
    Ok(())
}
```

### ب. استخدام سطر الأوامر (CLI)
```bash
# تشغيل البحث الدلالي في Exa
agent-reach --channel exa search "Max Weber legal rationalization"

# قراءة مخطوطة من قاعدة بيانات تراث
agent-reach --channel turath read --book 124 --page 45

# جلب خلاصة RSS
agent-reach --channel rss fetch "https://news.ycombinator.com/rss"
```

---

## 🛡️ 5. بوابات الجودة والاختبارات

مشروع محمي بـ 6 بوابات تحقق صارمة تتطلب نسبة نجاح 100%.

```bash
# تشغيل جميع اختبارات مساحة العمل (41/41 بوابة خضراء)
cargo test --workspace
```

- **`agent-reach-core`:** نجاح 10/10 اختبارات (بما فيها تحليل الوسائط بلغة Rust الخالصة).
- **`agent-reach-channels`:** نجاح 28/28 اختباراً.
- **`search_gauntlet`:** اعتماد 3/3 بوابات تحكيم.

---

## 👥 6. المساهمون

| الاسم / الهوية | الدور والمساهمات | الإحصائيات |
| :--- | :--- | :--- |
| **Ercan Er** | كبير المهندسين وصاحب المشروع (بنية Rust) | 38 commit، الشفرة البرمجية الأساسية |
| **Mihenk** | مدقق الشفرة وحارس بوابات التحكيم | اعتمادات التحكيم واختبارات Gauntlet |
| **El-Kassâm** | مطور الوكيل (MediaInspector والتكامل الخالص) | 12 commit، الوسائط وحزمة الاختبارات |
| **ZAI GLM 5.3** | مساهمات نموذج الوكيل وتعديل الكود | ذكاء النموذج |
| **GitHub Copilot** | إكمال البرمجة المساعد | مساعد البرمجة |
| **Hermes** | محرك إدارات الوكلاء | بيئة تشغيل الوكيل |

---

## 📄 7. الترخيص

مرخص بموجب **MIT License**. راجع `LICENSE` للمزيد من التفاصيل.
