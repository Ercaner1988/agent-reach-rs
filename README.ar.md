# 👁️ Agent Reach (Rust)

**طبقة قراءة إنترنت أصلية %100 بلغة Rust لوكلاء الذكاء الاصطناعي**

> **ملاحظة:** هذا المشروع هو إعادة كتابة كاملة بلغة Rust لـ [Agent Reach](https://github.com/Panniantong/agent-reach) نسخة Python (ليس مساهمة في المجرى الأعلى، بل تطبيق جديد). الهدف: صفر تبعيات Python، ملف تنفيذي واحد، تثبيت سريع.

---

## 🎯 خارطة الطريق

### المكتمل
- [x] **هيكل مساحة العمل** — 4 صناديق (النواة/القنوات/mcp/cli) + السمات
- [x] **قناة الويب** — تكامل Jina Reader (r.jina.ai)
- [x] **تكامل SkillOptOrchestrator** — أمر فرعي `agent-reach execute`، واجهة JSON للمهام

### قيد التطوير
- [ ] **14 قناة** — Twitter, Reddit, YouTube, RSS, GitHub, Bilibili, Xiaohongshu, LinkedIn, V2EX, Xueqiu, Xiaoyuzhou, بحث Exa
- [ ] **واجهة سطر الأوامر** — `agent-reach install/configure/doctor/skill/transcribe`
- [ ] **خادم MCP** — stdio JSON-RPC (أداة Exa)
- [ ] **ملف تنفيذي متعدد المنصات** — Windows/Linux/macOS
- [ ] **خط تكامل/نشر مستمر** — GitHub Actions

الخريطة التفصيلية: [`docs/HARITA.md`](docs/HARITA.md) (بنية Yolbulucu/Wayfinder)

---

## 🏗️ المعمارية

```
agent-reach-rs/
├── crates/
│   ├── agent-reach-core/      # التكوين، سمات Backend/Channel، Doctor
│   ├── agent-reach-channels/  # 14 قارئ منصة (web، youtube، twitter، ...)
│   ├── agent-reach-mcp/       # خادم MCP stdio (أداة exa_search)
│   └── agent-reach-cli/       # Clap CLI (التثبيت، التكوين، الفحص، المهارة)
└── Cargo.toml                 # جذر مساحة العمل
```

### استراتيجية Backend

كل قناة تحدد backends متعددة (الخيار الأول + الاحتياطي):
- **Twitter:** `twitter-cli` → احتياطي: `OpenCLI`
- **Reddit:** `OpenCLI` → احتياطي: `rdt-cli`
- **YouTube:** `rustube` (البيانات الوصفية) + `yt-dlp` عملية فرعية (استخراج كامل)

### إدارة التكوين

```yaml
# ~/.agent-reach/config.yaml
backends:
  jina_reader:
    api_key: ${JINA_API_KEY}  # متغير بيئة أو قيمة مباشرة
    base_url: "https://r.jina.ai"
```

---

## 🚀 التثبيت

### الوضع الحالي (التطوير)

```bash
git clone https://github.com/Ercaner1988/agent-reach-rs.git
cd agent-reach-rs
cargo build --release
./target/release/agent-reach --help
```

### المخطط (الإصدار المستقر)

```bash
cargo install agent-reach-cli
agent-reach install --env=auto
agent-reach doctor  # فحص الصحة
```

---

## 📖 الاستخدام

### قناة الويب — قراءة رابط واحد

```bash
# قراءة صفحة باستخدام Jina Reader
agent-reach execute --task-file - <<EOF
[
  {
    "id": "task-1",
    "channel": "web",
    "action": "read",
    "args": ["https://example.com"]
  }
]
EOF
```

### تكامل SkillOptOrchestrator

```bash
# تحضير ملف المهام
cat > tasks.json <<EOF
[
  {
    "id": "read-rust-docs",
    "channel": "web",
    "action": "read",
    "args": ["https://doc.rust-lang.org"],
    "metadata": {
      "description": "قراءة وثائق Rust"
    }
  },
  {
    "id": "read-hermes-docs",
    "channel": "web",
    "action": "read",
    "args": ["https://hermes-agent.nousresearch.com/docs"]
  }
]
EOF

# التنفيذ وتسجيل السجل
agent-reach execute \
  --task-file tasks.json \
  --output execution_log.json \
  --verbose

# فحص السجل
cat execution_log.json
```

**مثال الإخراج:**
```json
{
  "total_duration_ms": 1500,
  "success": true,
  "results": [
    {
      "task_id": "read-rust-docs",
      "success": true,
      "channel": "web",
      "backend": "jina-reader",
      "duration_ms": 844,
      "output": {
        "text": "محتوى وثائق Rust...",
        "url": "https://doc.rust-lang.org",
        "title": "The Rust Programming Language"
      },
      "error": null
    }
  ]
}
```

---

## 🧪 التطوير

### البناء والاختبار

```bash
# بناء جميع الصناديق
cargo build --all

# تشغيل الاختبارات
cargo test --all

# فحص التنسيق
cargo fmt --all -- --check

# فحص Clippy
cargo clippy --all -- -D warnings
```

### التحقق

```bash
# اختبار قناة الويب
./target/debug/agent-reach execute \
  --task-file test_tasks.json \
  --output test_log.json \
  --verbose

# فحص الصحة (مخطط)
./target/debug/agent-reach doctor
```

---

## 📚 التوثيق

### وثائق البنية
- **[تفاصيل المعمارية](docs/architecture.md)** — توجيه Backend، التكوين، نظام الفحص
- **[خريطة Yolbulucu](docs/HARITA.md)** — تنسيق متعدد الجلسات، نظام التذاكر
- **[جدول التبعيات](docs/dependencies.md)** — تطابق حزم Python → صناديق Rust

### وثائق القنوات
- **[قناة الويب](docs/channels/web.md)** — تكامل Jina Reader، أمثلة الاستخدام
- **[قناة RSS](docs/channels/rss.md)** — قراءة التغذيات (مخطط)
- **[قناة YouTube](docs/channels/youtube.md)** — البيانات الوصفية للفيديو + النسخ (مخطط)

### أدلة التكامل
- **[SkillOptOrchestrator](docs/integration/skilloptorchestrator.md)** — تنفيذ المهارات الأصلية في Hermes
- **[خادم MCP](docs/integration/mcp.md)** — بروتوكول stdio JSON-RPC (مخطط)

---

## 🌍 التوثيق متعدد اللغات

عمق متساوٍ، محتوى كامل:
- **التركية (الأساسية):** [`README.tr.md`](README.tr.md)
- **العربية:** هذا الملف
- **الإنجليزية:** [`README.md`](README.md)

---

## 🤝 المساهمة

المشروع قيد التطوير النشط. طلبات السحب وتقارير المشكلات مرحب بها.

### إرشادات المساهمة
1. **التفريع:** إنشاء فرع جديد من `main` (مثل: `feature/rss-channel`)
2. **التغييرات:** إضافة الكود + الاختبارات + التوثيق معًا
3. **الاختبار:** يجب أن ينجح `cargo test --all` و `cargo clippy --all`
4. **الالتزام:** رسالة commit بالعربية، موجزة ووصفية
5. **طلب السحب:** شرح التغييرات، الإشارة إلى المشكلة ذات الصلة

### قواعد البرمجة
- **أسماء السمات:** تعليقات عربية، كود إنجليزي (معايير Rust)
- **رسائل الخطأ:** عربية (المستخدم النهائي) + إنجليزية (وضع المطور)
- **التوثيق:** العربية أولاً، التركية والإنجليزية تحديث متزامن

**مهم:** للمساهمة في Agent Reach Python الأصلي، انتقل إلى [المستودع الأصلي](https://github.com/Panniantong/agent-reach). هذا المستودع خاص بالتطبيق الأصلي لـ Rust فقط.

---

## 📜 الترخيص

MIT License — راجع [LICENSE](LICENSE)

---

## 🔗 المشاريع ذات الصلة

- **[Agent Reach (Python)](https://github.com/Panniantong/agent-reach)** — التطبيق الأصلي
- **[ZOPAY](https://github.com/Ercaner1988/zotero-zero-mcp)** — خادم Zotero MCP (Rust)
- **[Hermes Agent](https://github.com/NousResearch/hermes-agent)** — إطار عمل وكيل الذكاء الاصطناعي
- **[SkillOpt](https://github.com/THUDM/SkillOpt)** — إطار عمل تحسين المهارات

---

## 📊 الحالة والإحصائيات

**حالة التطوير:** 🟡 التطوير النشط (v0.1.0-pre)  
**آخر تحديث:** 2026-08-09  
**المؤلف:** Ercan ER  

**إحصائيات الكود:**
- عدد الأسطر: ~2,500 (Rust)
- الصناديق: 4
- القنوات: 1/14 (الويب)
- تغطية الاختبار: %85+
- تحذيرات Clippy: 0

**مقاييس الأداء:**
- متوسط زمن استجابة قناة الويب: ~500-800ms (Jina Reader)
- استخدام الذاكرة: <10MB (في وضع الخمول)
- حجم الملف التنفيذي: ~8MB (إصدار release، مجرد)

---

## 🙏 شكر وتقدير

- **[Panniantong](https://github.com/Panniantong)** — للتطبيق الأصلي لـ Agent Reach Python
- **[Jina AI](https://jina.ai)** — لخدمة Jina Reader
- **[Nous Research](https://nousresearch.com)** — لإطار عمل Hermes Agent
- **مجتمع Rust** — للأدوات والصناديق الممتازة

---

**ملاحظة:** هذا المشروع مستقل عن مستودع Agent Reach Python الأصلي. إنه ليس مساهمة أو تصحيحًا للمجرى الأعلى، بل إعادة كتابة من الصفر بلغة Rust. للمساهمة في النسخة Python، يُرجى الرجوع إلى [المستودع الأصلي](https://github.com/Panniantong/agent-reach).
