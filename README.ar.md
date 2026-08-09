# 👁️ Agent Reach (Rust)

**طبقة قراءة إنترنت أصلية %100 بلغة Rust لوكلاء الذكاء الاصطناعي**

> **ملاحظة:** هذا المشروع هو إعادة كتابة كاملة بلغة Rust لـ [Agent Reach](https://github.com/Panniantong/agent-reach) نسخة Python (ليس مساهمة في المشروع الأصلي، بل تنفيذ جديد). الهدف: صفر تبعيات Python، ملف تنفيذي واحد، تثبيت سريع.

---

## 🎯 خارطة الطريق

- [x] **Workspace skeleton** — 4 crate (core/channels/mcp/cli) + trait'ler
- [x] **قناة الويب** — تكامل Jina Reader (r.jina.ai)
- [ ] **14 قناة** — Twitter, Reddit, YouTube, RSS, GitHub, Bilibili, Xiaohongshu, LinkedIn, V2EX, Xueqiu, Xiaoyuzhou, Exa Search
- [ ] **واجهة سطر الأوامر** — `agent-reach install/configure/doctor/skill/transcribe`
- [ ] **خادم MCP** — stdio JSON-RPC (أداة Exa)
- [ ] **SkillOptOrchestrator** — تكامل تنفيذ المهارات الأصلية في Hermes

الخريطة التفصيلية: [`docs/HARITA.md`](docs/HARITA.md) (معمارية Yolbulucu/Wayfinder)

---

## 🏗️ المعمارية

```
agent-reach-rs/
├── crates/
│   ├── agent-reach-core/      # Config, Backend/Channel traits, Doctor
│   ├── agent-reach-channels/  # 14 قارئ منصة (web, youtube, twitter, ...)
│   ├── agent-reach-mcp/       # خادم MCP stdio (أداة exa_search)
│   └── agent-reach-cli/       # Clap CLI (install, configure, doctor, skill)
└── Cargo.toml                 # جذر Workspace
```

**استراتيجية Backend:** كل قناة تحدد backends متعددة (الخيار الأول + الاحتياطي):
- **Twitter:** `twitter-cli` → احتياطي: `OpenCLI`
- **Reddit:** `OpenCLI` → احتياطي: `rdt-cli`
- **YouTube:** `rustube` (البيانات الوصفية) + `yt-dlp` subprocess (استخراج كامل)

---

## 🚀 التثبيت (مخطط)

```bash
cargo install agent-reach-cli
agent-reach install --env=auto
agent-reach doctor  # فحص الصحة
```

**حاليًا:** قيد التطوير. `cargo check --all` يعمل، تم تنفيذ القناة الأولى (Web).

---

## 🧪 التطوير

```bash
git clone https://github.com/Ercaner1988/agent-reach-rs.git
cd agent-reach-rs
cargo build --all
cargo test --all
cargo run --bin agent-reach-cli
```

---

## 📚 التوثيق

- **[تفاصيل المعمارية](docs/architecture.md)** — توجيه Backend، config، نظام doctor
- **[خريطة Yolbulucu](docs/HARITA.md)** — تنسيق متعدد الجلسات، نظام التذاكر
- **[جدول التبعيات](docs/dependencies.md)** — تطابق حزم Python → Rust crate

---

## 🌍 التوثيق متعدد اللغات

- **التركية (الأساسية):** [`README.tr.md`](README.tr.md)
- **العربية:** هذا الملف
- **الإنجليزية:** [`README.md`](README.md)

---

## 🤝 المساهمة

المشروع قيد التطوير النشط. PRs والـ issues مرحب بها.

**مهم:** للمساهمة في Agent Reach Python الأصلي، انتقل إلى [المستودع الأصلي](https://github.com/Panniantong/agent-reach). هذا المستودع خاص بالتنفيذ الأصلي لـ Rust فقط.

---

## 📜 الترخيص

MIT License — راجع [LICENSE](LICENSE)

---

## 🔗 المشاريع ذات الصلة

- [Agent Reach (Python)](https://github.com/Panniantong/agent-reach) — التنفيذ الأصلي
- [ZOPAY](https://github.com/Ercaner1988/zotero-zero-mcp) — خادم Zotero MCP (Rust)
- [Hermes Agent](https://github.com/NousResearch/hermes-agent) — إطار عمل وكيل الذكاء الاصطناعي

---

**الحالة:** 🟡 قيد التطوير (v0.1.0-pre)  
**آخر تحديث:** 2026-08-09  
**المؤلف:** Ercan ER
