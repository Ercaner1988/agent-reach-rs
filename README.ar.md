**🌍 [Türkçe](README.md) | [English](README.en.md) | [العربية](README.ar.md) | [日本語](README.ja.md) | [中文](README.zh.md) | [Русский](README.ru.md) | [Español](README.es.md)**

# Agent Reach RS (`agent-reach-rs`)

> **محرك قراءة مبني بالكامل بلغة Rust يمنح وكلاء الذكاء الاصطناعي عيوناً لرؤية الإنترنت بأكمله**

إن `agent-reach-rs` هو بيئة عمل (workspace) معيارية بلغة Rust تتيح لوكلاء الذكاء الاصطناعي (مثل Hermes، Claude، Codex، و OpenCode) قراءة البيانات بشكل موثوق من صفحات الويب، والشبكات الاجتماعية، والتشريعات وقواعد البيانات الأكاديمية، والملفات الصوتية والمرئية المحلية.

القياسات التي تستند إليها هذه الوثيقة: `git rev-parse --short HEAD` ← `dbe3b0c`، `cargo test --workspace` ← **58 اجتاز، 0 رسب، 1 تجاهل**، `harness/gates` ← **8 بوابات خضراء** (7 سبتمبر 2026).

---

## 🎯 1. الأهداف والميزات المكتملة

- **17 قارئ قنوات** — تحاول كل قناة استخدام واجهة خلفية (backend) واحدة أو أكثر؛ إذا فشلت إحداها، تتم تجربة الأخرى (تدرج: CLI ← API ← scraper).
- **لا يوجد اعتماد على ثنائي FFmpeg خارجي (`MediaInspector`):** يتم تحليل ملفات MP3، WAV، AAC، FLAC، OGG، و MKV باستخدام `symphonia` 0.5 دون أي ثنائي `ffmpeg` خارجي. حقل `format_name` يُرجع دائمًا `symphonia-native`.
- **مفتاح القنوات (`disabled_channels`):** يمكن إيقاف المصادر غير المرغوب فيها عبر الإعدادات أو متغير بيئة. تُنتج القناة المعطلة **خطأً مختلفًا عن القناة المعطلة تقنياً** — أحدهما ناتج عن قرار، والآخر خطأ يجب تتبعه.
- **التحقق من الحمولة (`require_payload`، ADR 0004):** يتم رصد الاستجابات التي تُرجع HTTP 200 دون محتوى حمولة فعلي (جدران تسجيل الدخول، إشعارات عدم الاتصال، صفحات 404) بدلاً من التعامل معها بصمت على أنها ناجحة.
- **بوابة ذهبية قبل الدفع (`.githooks/pre-push`):** تُشغل بوابات الجودة الـ 8 محليًا قبل الدفع (pre-push)؛ وتوقف العملية إذا فشلت فحوصات التنسيق أو البناء.
- **بوابة تكافؤ التكامل المستمر (`gate_ci_parity`):** تتحقق من أن جميع فحوصات `cargo` المحددة في إجراءات CI موجودة في بوابات اختبار البيئة (harness) المحلية.
- **نظام الكاسيت:** يتم تسجيل الاستجابات وإعادة تشغيلها؛ كما يتم تسجيل حالات الفشل أيضًا، بحيث يمكن اختبار المسار "غير المُقاس" بدون شبكة. متوقف افتراضيًا.
- **خادم MCP:** بروتوكول Model Context Protocol 2024-11-05، باستخدام stdio JSON-RPC، وخمس أدوات.
- **التفريغ الصوتي المحلي (`transcribe`):** يتم تفريغ ملف صوتي/مرئي محلي باستخدام Whisper (`whisper-large-v3-turbo`) من خلال Groq أو OpenAI.
- **قياس محمي بحكم:** تُحفظ المجموعة الذهبية، وعتبات القبول، وفحوصات التلاعب في ملفات منفصلة؛ لا يمكن لجهة القياس تغيير ما يتم قياسه.

### القنوات (17)

| Channel | Actions | Note |
| :--- | :--- | :--- |
| `web` | `read` | نص الصفحة من خلال Jina Reader |
| `rss` | `fetch`, `parse` | RSS 2.0 + Atom؛ منشورات Substack تستخدم أيضًا هذه القناة |
| `twitter` | `search`, `timeline`, `thread` | واجهتان خلفيتان |
| `youtube` | `metadata`, `transcript`, `search` | واجهتان خلفيتان |
| `github` | `repo`, `issue`, `pr`, `search` | `gh` CLI + REST API |
| `reddit` | `subreddit`, `search`, `post` | PRAW + REST API |
| `bilibili` | `video`, `info`, `search` | واجهتان خلفيتان |
| `xiaohongshu` | `note`, `search` | واجهة خلفية للويب |
| `linkedin` | `profile`, `company`, `search` | تتطلب بيانات اعتماد |
| `v2ex` | `topic`, `node`, `hot`, `latest` | Open API |
| `xueqiu` | `quote`, `stock`, `timeline`, `search` | بيانات السوق |
| `xiaoyuzhou` | `podcast`, `episode` | البودكاست |
| `exa` | `search` | بحث دلالي، يتطلب مفتاحًا |
| `duckduckgo` | `search` | مستخرج HTML، بدون مفتاح |
| `turath` | `search`, `book`, `author`, `page` | الأعمال الإسلامية الكلاسيكية؛ تُرجع **أرقام الأجزاء والصفحات** |
| `mevzuat` | `mevzuat`, `metin`, `fihrist` | التشريعات التركية (`mevzuat.gov.tr`)، يُستعلم عنها مباشرة برقم القانون |
| `uinjkt` | `identify`, `journals`, `articles`, `record` | المجلات الأكاديمية الإسلامية بجامعة UIN Jakarta ببروتوكول (OAI-PMH 2.0، وصول مفتوح) |

### المصادر المدرجة في قائمة الانتظار التالية

موثقة مع نقاط النهاية المقاسة في `docs/YOL-HARITASI-KAYNAKLAR.md`:
**UYAP Emsal** و **Yargıtay Karar Arama** (كلاهما JSON بدون مفتاح، `aramalist` + `getDokuman`)، و **Danıştay** (نقطة النهاية نشطة، وتُحل هيئة البحث)، و **Pew Research** (`robots.txt` يمنح صراحة وصول الوكيل، `ai-input=yes`)، و **Lexpera** (جدار اشتراك + تأخير زحف إلزامي Crawl-delay 10 ← نموذج البوابة البشرية)، بالإضافة إلى Wikipedia، و Stanford Encyclopedia of Philosophy، و TDV Encyclopedia of Islam، و OpenAlex، و Crossref، و المركز الوطني التركي للرسائل الجامعية.

---

## 🏗️ 2. البنية والوحدات

```text
agent-reach-rs/
├── Cargo.toml                    # Workspace (symphonia, tokio, reqwest, clap)
├── .githooks/                    # Rust-native git hooks (pre-push, post-commit, post-push)
├── crates/
│   ├── agent-reach-core/         # Channel, Backend, Config, Cassette, require_payload,
│   │                             #   MediaInspector, doctor, error
│   ├── agent-reach-channels/     # 17 channel implementations (including mevzuat, uinjkt, turath)
│   ├── agent-reach-mcp/          # MCP JSON-RPC server (stdio)
│   └── agent-reach-cli/          # Command-line client (binary: agent-reach)
├── harness/                      # Referee gates — built separately (outside workspace)
│   ├── kabul.json                # Acceptance thresholds (a file SEPARATE from the golden set)
│   └── biletler/                 # Tickets A · B · C and their outcomes
└── docs/
    ├── adr/                      # Architecture decisions 0001 · 0002 · 0003 · 0004
    └── YOL-HARITASI-KAYNAKLAR.md # Source generations and measured endpoints
```

**لماذا تقع `harness` خارج بيئة العمل:** إذا تم تجميع الأداة التي تُشغل البوابات مع الشجرة التي تقيسها، فإن أي كسر في التعليمات البرمجية سيؤدي إلى إسقاط الحكم معها.

### الأنواع الأساسية

| Type | Responsibility |
| :--- | :--- |
| `Channel` | اسم المصدر، وإجراءاته، والمنفذ |
| `Backend` | مسار واحد تحت القناة؛ يبلغ عن الحالة عبر `BackendStatus` |
| `Config` | يقرأ/يكتب `~/.agent-reach/config.yaml`، ويفرض `channel_enabled` |
| `Cassette` | تسجيل الاستجابات وإعادة تشغيلها (`AGENT_REACH_CASSETTE`) |
| `MediaInspector` | محلل وسائط بلغة Rust الخالصة (`symphonia`) |
| `HealthCheck` | نوع البيانات خلف مخرجات أمر `doctor` |

---

## 🚀 3. التثبيت والتكوين

### المتطلبات الأساسية

- **Rust 1.75+** (`cargo` و `rustc`).
- **لا يلزم استخدام ثنائيات خارجية:** FFmpeg و Python و Node.js ليست إلزامية. يتم استخدام الواجهات الخلفية الاختيارية لبعض القنوات (`gh` CLI، `praw`، `BBDown`) عند تثبيتها؛ وإلا تنتقل القناة إلى واجهتها الخلفية الأخرى.

### البناء

```bash
git clone https://github.com/Ercaner1988/agent-reach-rs.git
cd agent-reach-rs
cargo build --release
```

الثنائي المنتج: `target/release/agent-reach` (`agent-reach.exe` على نظام Windows).

> **ملاحظة:** لا ينشر هذا المستودع ثنائيات معدة مسبقًا وقابلة للتنزيل. يقوم `dist-workspace.toml` بتعيين `installers = []`؛ يتم إنتاج أصول الإصدار كأرشيفات فقط. البناء من المصدر هو المسار الوحيد.

### التكوين لأول مرة

```bash
agent-reach install                      # Creates the ~/.agent-reach directory
agent-reach configure github_token <tk>  # Set a key
agent-reach configure --json             # View current settings
agent-reach doctor                       # Channel health check
agent-reach doctor --json                # Machine-readable health report
```

الأمر `install` **لا** يُثبت أدوات خارجية؛ بل يجهز دليل التكوين فقط.

### تعطيل القنوات

`~/.agent-reach/config.yaml`:

```yaml
disabled_channels:
  - reddit
  - linkedin
```

أو، لتشغيل لمرة واحدة دون المساس بالملف:

```bash
AGENT_REACH_DISABLED_CHANNELS=reddit,linkedin agent-reach execute --task-file tasks.json
```

تتجاهل مقارنة الأسماء حالة الأحرف والمسافات البيضاء؛ ولا توجد مطابقة للبادئة (كتابة `red` لن تُعطل `reddit`). يُطبق ذلك على مساري CLI و MCP.

### مفاتيح التكوين المعترف بها

`twitter_auth_token` · `twitter_ct0` · `groq_api_key` · `openai_api_key` · `github_token` · `reddit_client_id` · `reddit_client_secret` · `reddit_user_agent` · `linkedin_username` · `linkedin_password` · `exa_api_key` · `proxy` · `disabled_channels`

---

## 📖 4. الاستخدام والأمثلة

### أ. سطر الأوامر

الأوامر الفرعية من `agent-reach --help`:

```text
install     Set up the config directory (external tools are not installed)
configure   Set a config value or auto-extract from browser
doctor      Check platform availability
skill       Manage agent skill registration
execute     Execute tasks from a JSON file
transcribe  Transcribe a local audio/video file
```

### ب. تنفيذ المهام

يتم توجيه القنوات من خلال ملف المهام (جميع القنوات الـ 17 مدعومة عبر CLI):

```json
[
  {
    "id": "task-1",
    "channel": "turath",
    "action": "search",
    "args": ["الاجتهاد"]
  },
  {
    "id": "task-2",
    "channel": "mevzuat",
    "action": "mevzuat",
    "args": ["5237"]
  },
  {
    "id": "task-3",
    "channel": "uinjkt",
    "action": "journals",
    "args": []
  }
]
```

```bash
agent-reach execute --task-file tasks.json --output log.json
agent-reach execute --task-file tasks.json --continue-on-error --verbose
```

مخطط (schema) المخرجات (`log.json`):

```json
{
  "total_duration_ms": 1240,
  "success": true,
  "results": [
    {
      "task_id": "task-1",
      "success": true,
      "channel": "turath",
      "backend": "turath-api",
      "duration_ms": 812,
      "output": "…",
      "error": null
    }
  ]
}
```

### ج. التفريغ الصوتي المحلي

```bash
agent-reach transcribe recording.mp3
agent-reach transcribe lecture.mkv --provider groq --output lecture.txt
```

يتم قبول الملفات المحلية فقط. في حال عدم تكوين `groq_api_key` ولا `openai_api_key`، يوضح الأمر ذلك بصراحة ويوجهك إلى إعدادات `configure`.

### د. تحليل الوسائط بلغة Rust الخالصة (`MediaInspector`)

```rust
use agent_reach_core::MediaInspector;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let meta = MediaInspector::inspect_file("sample_audio.mp3")?;

    println!("Format: {}", meta.format_name);          // symphonia-native
    println!("Codec: {}", meta.codec_name);
    println!("Sample rate: {} Hz", meta.sample_rate);
    println!("Channels: {}", meta.channels);
    println!("Duration: {:.2} seconds", meta.duration_seconds);

    Ok(())
}
```

### هـ. خادم MCP

يتواصل الثنائي `agent-reach-mcp` بصيغة JSON-RPC عبر مسار stdio (بروتوكول `2024-11-05`) ويعرض خمس أدوات:

| Tool | Parameters |
| :--- | :--- |
| `web_read` | `url` |
| `rss_fetch` | `url` |
| `rss_parse` | `xml` |
| `exa_search` | `query`, `num_results` (1–10) |
| `agent_reach_execute` | `channel`, `action`, `args[]` |

تقبل الأداة `agent_reach_execute` هذه القنوات الاثني عشر: `bilibili`، `duckduckgo`، `github`، `linkedin`، `reddit`، `turath`، `twitter`، `v2ex`، `xiaohongshu`، `xiaoyuzhou`، `xueqiu`، و `youtube`. بينما يتم الوصول إلى `web`، `rss`، و `exa` من خلال أدواتهم الخاصة — لذا تتوفر 14 قناة عبر خادم MCP.

> **ملاحظة الأمانة:** بينما تعمل جميع القنوات الـ 17 من خلال `execute` الخاص بـ CLI، لم يتم بعد إضافة `mevzuat` و `uinjkt` إلى تعداد `agent_reach_execute` الخاص بـ MCP أو حتى إلى أمر `doctor`؛ وهي مجدولة للإصدار القادم.

تكوين سطح مكتب Hermes / Claude:

```json
{
  "mcpServers": {
    "agent-reach": {
      "command": "/install/path/agent-reach-mcp"
    }
  }
}
```

---

## 🛡️ 5. بوابات الجودة والاختبارات

### حالة الاختبارات المقاسة

```bash
cargo test --workspace
```

تم التشغيل على هذا المستودع في 7 سبتمبر 2026، الإخراج الفعلي:

| Suite | Result |
| :--- | :--- |
| `agent_reach_channels` (lib) | 39 اجتاز · 0 رسب |
| `agent_reach_core` (lib) | 16 اجتاز · 0 رسب |
| `search_gauntlet` (integration) | 3 اجتاز · 0 رسب · 1 تجاهل |
| **Total** | **58 اجتاز · 0 رسب · 1 تجاهل** |

يتطلب الاختبار الوحيد المتجاهل (ignored) شبكة حية؛ قم بتشغيله يدويًا عبر استخدام `--ignored`.

### نظام الحكم وبوابات الجودة الـ 8 الذهبية

إن `harness` عبارة عن حزمة منفصلة تُشغل 8 بوابات عبر أمر `cargo run --manifest-path harness/Cargo.toml -- gates`:

1. **build** — التحقق من صحة بناء بيئة العمل.
2. **clippy** — تطبيق مبدأ خلو الكود من التحذيرات (`-D warnings`).
3. **unit tests** — اجتياز جميع اختبارات الوحدة.
4. **formatting** — الامتثال لتنسيق `cargo fmt`.
5. **answer-key grep** — ضمان عدم نسخ سلاسل إجابات المجموعة الذهبية وتسريبها إلى التعليمات البرمجية المصدرية (تم مسح 138 عبارة).
6. **referee watch** — التحقق من عدم تعديل ملفات الحكم.
7. **criteria wiring** — تأكيد أن معايير القبول يتم فحصها بنشاط.
8. **ci parity** — ضمان وجود جميع فحوصات CI في بوابات البيئة (harness) المحلية.

عتبات القبول موجودة في `harness/kabul.json`، وهي **منفصلة** عن المجموعة الذهبية (`min_recall_ratio: 0.9`، `max_zero_results: 0`). تحتوي المجموعة الذهبية على **24 حالة** في مسار `crates/agent-reach-channels/tests/golden_search.json`.

**فجوة مسجلة بصراحة:** تستهدف جميع الحالات الأربع والعشرين قناة `github`. شرط التذكرة أ (Ticket A) — "لا تُعتبر القناة الجديدة عاملة حتى تفوز بحالة واحدة على الأقل في المجموعة الذهبية" — **لم يتحقق بعد** بالنسبة لـ `turath`، `mevzuat`، و `uinjkt`. تم قياس هذه القنوات مباشرة، لكنها لم تدخل المجموعة الذهبية بعد.

### أخلاقيات القياس

- وفقًا لـ `docs/adr/0003-a-refusal-is-not-a-verdict.md`: إن رفض طبقة النقل (HTTP 429, 202) **لا يُحسب على أنه إخفاق في القدرة**؛ بل يُصنف على أنه `Not measured` (غير مُقاس).
- وفقًا لـ `docs/adr/0004-a-200-is-not-a-payload-verdict.md`: استجابات HTTP 200 التي لا تحتوي على بيانات حقيقية (`require_payload`) تُعامل كأخطاء.

---

## 👥 6. المساهمون

الأعداد أدناه مأخوذة من `git shortlog -sne --all` (7 سبتمبر 2026، بإجمالي 77 التزامًا).

| Name / Identity | Role and Contributions | Measurement |
| :--- | :--- | :--- |
| **Ercan ER** ([@Ercaner1988](https://github.com/Ercaner1988)) | مالك المشروع والمهندس الرئيسي؛ معمارية Rust، تصميم القنوات الـ 17، الأجيال المصدرية، أخلاقيات القياس (ADR 0001–0004) | **74 التزامًا** (43 + 22 + 9، 3 توقيعات مدمجة) |
| **copilot-swe-agent[bot]** ([@copilot-swe-agent[bot]](https://github.com/apps/copilot-swe-agent)) | طلب السحب PR #2: فحوصات CI `cargo fmt` وإصلاحات لتنسيقات lint-and-format | **2 التزامات** |
| **Devin AI** ([@devin-ai-integration[bot]](https://github.com/apps/devin-ai-integration)) | طلب السحب PR #1: اكتشاف مسار Python عبر الأنظمة، ضمانات لمنع الحقن في LinkedIn/Reddit، توجيه CLI | **1 التزام** |
| **Kassam** (Hermes agent) | مطور نظير من الذكاء الاصطناعي؛ أداة `MediaInspector`، قناة `turath`، مفتاح القنوات، ومجموعة الوثائق بسبع لغات | نُفذت تحت توقيع Ercan ER |
| **Mihenk** (Claude Opus 5) | مراجع البنية الهندسية والحكم؛ مقررات ADR 0001–0004، قفل الحكم، التذاكر A · B · C، والتحقق من الحمولة | نُفذت تحت توقيع Ercan ER |
| **GitHub Copilot** (Editor Assistant) | إكمال التعليمات البرمجية داخل المحرر؛ أنماط Rust المتكررة، أذرع `match`، وسقالات الاختبار | لا يوجد توقيع git منفصل |
| **ZAI GLM 5.3** (Zhipu AI) | تدرج البحث واقتراحات لتحسين القنوات (الالتزام `139b713`) | نُفذت تحت توقيع Ercan ER |

**ملاحظة الأمانة:** Kassam، و Mihenk، و ZAI GLM 5.3، ومساعد التحرير GitHub Copilot لا يقومون بالالتزام تحت توقيعات git منفصلة؛ إذ تُسجل تغييراتهم تحت توقيع Ercan ER. في حين أن روبوتات GitHub المباشرة التي تفتح طلبات السحب (`copilot-swe-agent[bot]`: 2، `Devin AI`: 1) تظهر بتوقيعاتها الخاصة.

---

## 📄 7. الترخيص

هذا المشروع مرخص بموجب **MIT License** — انظر ملف `LICENSE`.
حقوق الطبع والنشر: 2026 Ercan ER.

التوثيق: `CONTEXT.md` (مسرد المصطلحات)، `docs/adr/` (قرارات معمارية 0001–0004)، `docs/YOL-HARITASI-KAYNAKLAR.md` (الأجيال المصدرية)، `docs/channels/mevzuat.md`، `docs/channels/KANAL-EKLEME.md`، `docs/integration/mcp.md`، و `docs/channels/rss.md`.
