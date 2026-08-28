**🌍 [Türkçe](README.md) | [English](README.en.md) | [العربية](README.ar.md) | [日本語](README.ja.md)**

---

# 👁️ Agent Reach (Rust)

**طبقة قراءة إنترنت وبحث دلالي أصلية %100 بلغة Rust لوكلاء الذكاء الاصطناعي**

> **ملاحظة:** هذا المشروع هو إعادة كتابة كاملة بلغة Rust لـ [Agent Reach](https://github.com/Panniantong/agent-reach) نسخة Python (تطبيق جديد ومستقل بالكامل). الهدف: صفر تبعيات Python، ملف تنفيذي واحد، تجميع بلغة Rust الخالصة، سرعة فائقة.

---

## 🎯 خارطة الطريق والميزات المكتملة

### المكونات الأساسية المكتملة
- [x] **هيكل مساحة العمل** — 4 صناديق (`core` و`channels` و`mcp` و`cli`)
- [x] **قناة الويب** — تكامل Jina Reader (`r.jina.ai`)
- [x] **قناة RSS** — جلب وتحليل تغذيات RSS 2.0 وAtom (`fetch` + `parse`)
- [x] **تويتر** — `twitter-cli` (مصادقة)، Nitter (مجهول)
- [x] **يوتيوب** — `yt-dlp` (بيانات وصفية، نص الفيديو، بحث)
- [x] **GitHub** — `gh` CLI (سلم التخفيف)، GitHub REST API
- [x] **Reddit** — Reddit API (OAuth2)، PRAW (Python)
- [x] **وسائل التواصل الصينية والمالية** — Bilibili و Xiaohongshu و V2EX و Xueqiu و Xiaoyuzhou
- [x] **المهنية والبحث** — LinkedIn و Exa Search و DuckDuckGo (بحث HTML)
- [x] **واجهة سطر الأوامر** — `install` و`configure` و`doctor` و`skill` و`transcribe` و`execute`
- [x] **خادم MCP** — stdio JSON-RPC مع 5 أدوات (`web_read` و`rss_fetch` و`rss_parse` و`exa_search` و`agent_reach_execute`)
- [x] **التجميع متعدد المنصات** — Windows و Linux و macOS (`cargo-dist`)
- [x] **خط النشر والتكامل المستمر** — GitHub Actions CI/CD وبوابات الاختبار الآلية (`harness/` (Rust))

موارد خارطة الطريق: [`docs/YOL-HARITASI-KAYNAKLAR.md`](docs/YOL-HARITASI-KAYNAKLAR.md)

### القيود المعروفة (غير منفذة بعد)
- صندوق `agent-reach-graph` (الخريطة الذهنية الدلالية) مخطط له؛ غير موجود في المستودع بعد.
- خلفية `rustube` لليوتيوب هي عنصر نائب؛ المسار العامل هو عملية `yt-dlp` الفرعية.
- احتياطي Nitter لتويتر يقوم باستخراج HTML بسيط على مستوى العنصر النائب.
- `configure --from-browser` (استخراج الكوكيز من المتصفح) غير منفذ.
- `install` يجهز دليل التكوين فقط؛ لا يثبت الأدوات الخارجية مثل `gh` و`yt-dlp` و`twitter-cli`.
- يتطلب Reddit بيانات اعتماد OAuth2 (`reddit_client_id` و`reddit_client_secret`).

---

## 🏗️ المعمارية

```
agent-reach-rs/
├── crates/
│   ├── agent-reach-core/      # التكوين، سمات Backend/Channel، التخزين المؤقت للشريط
│   ├── agent-reach-channels/  # 15 قارئ منصة (web، youtube، twitter، github، ...)
│   ├── agent-reach-mcp/       # خادم MCP stdio JSON-RPC
│   └── agent-reach-cli/       # Clap CLI (التثبيت، الفحص، المهارة، التنفيذ)
├── harness/                   # بوابات الفحص الآلي والتخزين المؤقت
└── Cargo.toml                 # جذر مساحة العمل
```

### استراتيجية Backend

كل قناة تحدد backends متعددة (الخيار الأول + الاحتياطي):
- **Twitter:** `twitter-cli` $\rightarrow$ احتياطي: `Nitter`
- **Reddit:** `Reddit API` $\rightarrow$ احتياطي: `PRAW`
- **YouTube:** `yt-dlp` عملية فرعية (بيانات وصفية، نص الفيديو، بحث)؛ خلفية `rustube` عنصر نائب
- **GitHub:** `gh` CLI (تقسيم المصطلحات بدون علامات تنصيص) $\rightarrow$ احتياطي: `GitHub REST API`

---

## 🚀 التثبيت

### التجميع من المصدر

```bash
git clone https://github.com/Ercaner1988/agent-reach-rs.git
cd agent-reach-rs
cargo build --release
./target/release/agent-reach --help
```

### التثبيت المستقر

```bash
cargo install agent-reach-cli
agent-reach install --env=auto
agent-reach doctor  # فحص الصحة والتبعيات
```

---

## 📖 الاستخدام والتنفيذ

### قناة الويب — قراءة صفحة واحدة

```bash
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

### تنفيذ المهام الجماعية (`tasks.json`)

```bash
cat > tasks.json <<EOF
[
  {
    "id": "read-rust-docs",
    "channel": "web",
    "action": "read",
    "args": ["https://doc.rust-lang.org"]
  },
  {
    "id": "search-github",
    "channel": "github",
    "action": "search",
    "args": ["http client library"]
  }
]
EOF

agent-reach execute --task-file tasks.json --output execution_log.json --verbose
```

---

## 🛡️ بوابات الاختبار الآلية

تخضع كل إضافية جديدة للمشروع لـ 6 بوابات اختبار مجانية (`harness/` (Rust)):

```bash
cargo run --manifest-path harness/Cargo.toml -- gates
```

- **البوابة 1 (التجميع):** `cargo build --workspace`
- **البوابة 2 (Clippy):** `cargo clippy --workspace --all-targets -- -D warnings`
- **البوابة 3 (اختبارات الوحدة):** `cargo test --workspace`
- **البوابة 4 (التنسيق):** `cargo fmt --check`
- **البوابة 5 (فحص الإجابات):** فحص آلي لمنع تسرب كلمات الإجابة إلى الكود
- **البوابة 6 (حارس الحد):** التحقق من مرجع git لملفات الحكم

---

## 👥 المساهمون

للحصول على القائمة التفصيلية، يرجى مراجعة ملف [`CONTRIBUTORS.md`](CONTRIBUTORS.md).
- **Ercan ER** ([@Ercaner1988](https://github.com/Ercaner1988)) — مالك المشروع وكبير المهندسين
- **Kassam** (Hermes Agent / Nous Research) — زميل الذكاء الاصطناعي والمطور المشارك
- **Mihenk** (Claude Opus 5 / Anthropic) — الحكم والمراجع المعماري
- **Devin AI** — المطور الآلي المساهم
