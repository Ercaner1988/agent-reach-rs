**🌍 [Türkçe](README.md) | [English](README.en.md) | [العربية](README.ar.md) | [日本語](README.ja.md) | [中文](README.zh.md) | [Русский](README.ru.md) | [Español](README.es.md)**

# Agent Reach RS (`agent-reach-rs`)

> **Механизм чтения полностью на Rust, который дает ИИ-агентам возможность видеть весь интернет**

`agent-reach-rs` — это модульное рабочее пространство Rust, которое позволяет ИИ-агентам (Hermes, Claude, Codex, OpenCode) надежно считывать данные с веб-страниц, социальных сетей, законодательных и академических баз данных, а также локальных аудио/видео файлов.

Измерения, на которых базируется этот документ: `git rev-parse --short HEAD` → `dbe3b0c`, `cargo test --workspace` → **58 passed, 0 failed, 1 ignored**, `harness/gates` → **8 gates green** (7 September 2026).

---

## 🎯 1. Цели и реализованные функции

- **17 channel readers** — каждый канал (channel) пробует один или несколько бэкендов; если один выходит из строя, пробуется следующий (CLI → API → парсер скрапер).
- **Отсутствие внешней зависимости FFmpeg (`MediaInspector`):** MP3, WAV, AAC, FLAC, OGG и MKV парсятся с помощью `symphonia` 0.5 без внешнего бинарного файла `ffmpeg`. Поле `format_name` всегда возвращает `symphonia-native`.
- **Переключатель каналов (`disabled_channels`):** Нежелательные источники можно отключить через конфигурацию или переменную окружения. Отключенный канал выдает **ошибку, отличную от сломанного канала** — одно является решением, другое — неисправностью, которую нужно искать.
- **Проверка полезной нагрузки (`require_payload`, ADR 0004):** Ответы, возвращающие HTTP 200 без фактического содержимого полезной нагрузки (окна авторизации, сообщения об оффлайн-режиме, 404 страницы-заглушки), перехватываются и не рассматриваются молча как успешные.
- **Золотой шлюз перед отправкой (pre-push golden gate, `.githooks/pre-push`):** Локально запускает 8 шлюзов качества (quality gates) перед отправкой (push); останавливает push, если проверки форматирования или сборки завершаются неудачей.
- **Шлюз паритета CI (`gate_ci_parity`):** Проверяет, что все проверки `cargo`, определенные в рабочих процессах CI, присутствуют в локальных тестовых шлюзах harness.
- **Кассетная система (Cassette):** Ответы записываются и воспроизводятся; сбои также записываются, поэтому путь "не измерено" (not measured) можно проверить без сети. По умолчанию отключено.
- **MCP сервер:** Model Context Protocol 2024-11-05, stdio JSON-RPC, пять инструментов.
- **Локальная транскрипция (`transcribe`):** Локальный аудио/видео файл транскрибируется с помощью Whisper (`whisper-large-v3-turbo`) через Groq или OpenAI.
- **Измерение, защищенное судьей (referee):** Золотой набор (golden set), пороги приемки и проверки на вмешательство находятся в отдельных файлах; измеряющая сторона не может изменять измеряемое.

### Каналы (17 channels)

| Channel | Actions | Note |
| :--- | :--- | :--- |
| `web` | `read` | Текст страницы через Jina Reader |
| `rss` | `fetch`, `parse` | RSS 2.0 + Atom; публикации Substack также используют этот канал |
| `twitter` | `search`, `timeline`, `thread` | Два бэкенда |
| `youtube` | `metadata`, `transcript`, `search` | Два бэкенда |
| `github` | `repo`, `issue`, `pr`, `search` | CLI `gh` + REST API |
| `reddit` | `subreddit`, `search`, `post` | PRAW + REST API |
| `bilibili` | `video`, `info`, `search` | Два бэкенда |
| `xiaohongshu` | `note`, `search` | Веб-бэкенд |
| `linkedin` | `profile`, `company`, `search` | Требует учетные данные |
| `v2ex` | `topic`, `node`, `hot`, `latest` | Open API |
| `xueqiu` | `quote`, `stock`, `timeline`, `search` | Рыночные данные |
| `xiaoyuzhou` | `podcast`, `episode` | Подкасты |
| `exa` | `search` | Семантический поиск, требует ключ |
| `duckduckgo` | `search` | Извлекатель HTML, без ключа |
| `turath` | `search`, `book`, `author`, `page` | Классические исламские труды; возвращает **номера томов и страниц** |
| `mevzuat` | `mevzuat`, `metin`, `fihrist` | Законодательство Турции (`mevzuat.gov.tr`), запрашивается напрямую по номеру закона |
| `uinjkt` | `identify`, `journals`, `articles`, `record` | Исламские академические журналы UIN Jakarta (OAI-PMH 2.0, открытый доступ) |

### Источники в очереди

Документированы с измеренными эндпоинтами в `docs/YOL-HARITASI-KAYNAKLAR.md`:
**UYAP Emsal** и **Yargıtay Karar Arama** (оба безключевой JSON, `aramalist` + `getDokuman`), **Danıştay** (эндпоинт живой, нужно разобрать тело поиска), **Pew Research** (`robots.txt` явно разрешает доступ агентам, `ai-input=yes`), **Lexpera** (подписка + обязательная задержка Crawl-delay 10 секунд → модель с человеческим участием), плюс Wikipedia, Stanford Encyclopedia of Philosophy, TDV Encyclopedia of Islam, OpenAlex, Crossref и Национальный центр диссертаций Турции (Turkish National Thesis Center).

---

## 🏗️ 2. Архитектура и модули

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

**Почему `harness` находится вне workspace:** если инструмент, который запускает тесты (gates), компилируется вместе с деревом, которое он измеряет, то поломка в коде сломает и самого судью.

### Основные типы (Core types)

| Type | Responsibility |
| :--- | :--- |
| `Channel` | Название источника, действия (actions) и исполнитель |
| `Backend` | Единый путь внутри канала; сообщает о здоровье через `BackendStatus` |
| `Config` | Читает/пишет `~/.agent-reach/config.yaml`, обеспечивает соблюдение `channel_enabled` |
| `Cassette` | Запись и воспроизведение ответов (`AGENT_REACH_CASSETTE`) |
| `MediaInspector` | Парсер медиа полностью на Rust (`symphonia`) |
| `HealthCheck` | Тип данных, стоящий за выводом `doctor` |

---

## 🚀 3. Установка и настройка

### Предварительные требования

- **Rust 1.75+** (`cargo` и `rustc`).
- **Не требуются внешние бинарные файлы:** FFmpeg, Python и Node.js не обязательны. Дополнительные бэкенды для определенных каналов (CLI `gh`, `praw`, `BBDown`) используются, если они установлены; в противном случае канал переключается на свой другой бэкенд.

### Сборка

```bash
git clone https://github.com/Ercaner1988/agent-reach-rs.git
cd agent-reach-rs
cargo build --release
```

Создаваемый бинарный файл: `target/release/agent-reach` (`agent-reach.exe` на Windows).

> **Примечание:** Этот репозиторий не публикует предварительно собранные загружаемые бинарные файлы (prebuilt binaries). `dist-workspace.toml` устанавливает `installers = []`; релизные ассеты производятся только в виде архивов. Сборка из исходного кода — единственный путь.

### Первоначальная настройка

```bash
agent-reach install                      # Creates the ~/.agent-reach directory
agent-reach configure github_token <tk>  # Set a key
agent-reach configure --json             # View current settings
agent-reach doctor                       # Channel health check
agent-reach doctor --json                # Machine-readable health report
```

Команда `install` **не** устанавливает внешние инструменты; она лишь подготавливает каталог конфигурации.

### Отключение каналов

`~/.agent-reach/config.yaml`:

```yaml
disabled_channels:
  - reddit
  - linkedin
```

Или, для одного запуска без изменения файла:

```bash
AGENT_REACH_DISABLED_CHANNELS=reddit,linkedin agent-reach execute --task-file tasks.json
```

Сравнение названий игнорирует регистр и пробелы; сопоставление по префиксу отсутствует (написание `red` не отключит `reddit`). Это применяется как к пути CLI, так и к MCP.

### Распознаваемые ключи конфигурации (config keys)

`twitter_auth_token` · `twitter_ct0` · `groq_api_key` · `openai_api_key` · `github_token` · `reddit_client_id` · `reddit_client_secret` · `reddit_user_agent` · `linkedin_username` · `linkedin_password` · `exa_api_key` · `proxy` · `disabled_channels`

---

## 📖 4. Использование и примеры

### A. Командная строка

Подкоманды из `agent-reach --help`:

```text
install     Set up the config directory (external tools are not installed)
configure   Set a config value or auto-extract from browser
doctor      Check platform availability
skill       Manage agent skill registration
execute     Execute tasks from a JSON file
transcribe  Transcribe a local audio/video file
```

### B. Выполнение задач (tasks)

Каналы управляются через файл задач (все 17 каналов поддерживаются через CLI):

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

Схема вывода (`log.json`):

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

### C. Локальная транскрипция

```bash
agent-reach transcribe recording.mp3
agent-reach transcribe lecture.mkv --provider groq --output lecture.txt
```

Принимаются только локальные файлы. Если ни `groq_api_key`, ни `openai_api_key` не настроены, команда прямо сообщает об этом и указывает на `configure`.

### D. Парсинг медиа полностью на Rust (`MediaInspector`)

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

### E. Сервер MCP

Бинарный файл `agent-reach-mcp` общается по JSON-RPC через stdio (protocol 2024-11-05) и предоставляет пять инструментов (tools):

| Tool | Parameters |
| :--- | :--- |
| `web_read` | `url` |
| `rss_fetch` | `url` |
| `rss_parse` | `xml` |
| `exa_search` | `query`, `num_results` (1–10) |
| `agent_reach_execute` | `channel`, `action`, `args[]` |

`agent_reach_execute` принимает эти двенадцать каналов (channels): `bilibili`, `duckduckgo`, `github`, `linkedin`, `reddit`, `turath`, `twitter`, `v2ex`, `xiaohongshu`, `xiaoyuzhou`, `xueqiu`, `youtube`. `web`, `rss` и `exa` доступны через свои собственные инструменты — таким образом, по MCP доступно 14 каналов.

> **Заметка о честности (Honesty note):** В то время как все 17 каналов работают через CLI `execute`, `mevzuat` и `uinjkt` еще не были добавлены в перечисление MCP `agent_reach_execute` или в команду `doctor`; их добавление запланировано на следующий релиз.

Настройка Hermes / Claude Desktop:

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

## 🛡️ 5. Контроль качества и тесты

### Статус измеренных тестов

```bash
cargo test --workspace
```

Запущено для этого репозитория 7 September 2026, фактический вывод:

| Suite | Result |
| :--- | :--- |
| `agent_reach_channels` (lib) | 39 passed · 0 failed |
| `agent_reach_core` (lib) | 16 passed · 0 failed |
| `search_gauntlet` (integration) | 3 passed · 0 failed · 1 ignored |
| **Total** | **58 passed · 0 failed · 1 ignored** |

Единственный пропущенный тест (ignored test) требует активного сетевого соединения; запустите его вручную с флагом `--ignored`.

### Судейская система и 8 золотых шлюзов

`harness` — это отдельный контейнер (crate), который запускает 8 шлюзов (golden gates) через `cargo run --manifest-path harness/Cargo.toml -- gates`:

1. **build** — Валидация компиляции рабочего пространства (workspace).
2. **clippy** — Обеспечение отсутствия предупреждений (`-D warnings`).
3. **unit tests** — Успешное прохождение всех модульных тестов.
4. **formatting** — Соответствие форматированию `cargo fmt`.
5. **answer-key grep** — Гарантирует, что строки ответов из золотого набора (golden-set) не скопированы в исходный код (просканировано 138 фраз).
6. **referee watch** — Проверяет, что файлы судьи (referee files) не были изменены.
7. **criteria wiring** — Подтверждает, что критерии приемки активно проверяются.
8. **ci parity** — Гарантирует, что все проверки CI существуют в локальных шлюзах harness.

Пороги (Thresholds) находятся в `harness/kabul.json`, **отдельно** от золотого набора (`min_recall_ratio: 0.9`, `max_zero_results: 0`). Золотой набор (golden set) содержит **24 cases** в `crates/agent-reach-channels/tests/golden_search.json`.

**Открыто зафиксированный пробел (A gap recorded openly):** все 24 золотых случая (24 cases) направлены на `github`. Условие Билета A (Ticket A) — "новый канал не считается рабочим, пока он не выиграет хотя бы один случай в золотом наборе" — **еще не выполнено** для `turath`, `mevzuat` и `uinjkt`. Каналы были измерены в реальном времени, но не вошли в золотой набор (golden set).

### Этика измерений (Measurement ethics)

- Согласно `docs/adr/0003-a-refusal-is-not-a-verdict.md`: отказ на транспортном уровне HTTP (429, 202) **не считается отсутствием возможности**; он помечается как `Not measured`.
- Согласно `docs/adr/0004-a-200-is-not-a-payload-verdict.md`: ответы HTTP 200 без реальных данных (`require_payload`) рассматриваются как ошибки.

---

## 👥 6. Участники (Contributors)

Показатели ниже получены из `git shortlog -sne --all` (7 September 2026, всего 77 commits).

| Name / Identity | Role and Contributions | Measurement |
| :--- | :--- | :--- |
| **Ercan ER** ([@Ercaner1988](https://github.com/Ercaner1988)) | Владелец проекта и ведущий архитектор; архитектура Rust, дизайн для 17 каналов, генерации источников, этика измерений (Measurement ethics, ADR 0001–0004) | **74 commits** (43 + 22 + 9, three signatures merged) |
| **copilot-swe-agent[bot]** ([@copilot-swe-agent[bot]](https://github.com/apps/copilot-swe-agent)) | PR #2: CI исправления форматирования `cargo fmt` и lint-and-format | **2 commits** |
| **Devin AI** ([@devin-ai-integration[bot]](https://github.com/apps/devin-ai-integration)) | PR #1: кроссплатформенное определение пути Python, защиты от внедрения в LinkedIn/Reddit, маршрутизация CLI | **1 commit** |
| **Kassam** (Hermes agent) | Ко-разработчик ИИ; `MediaInspector`, канал `turath`, переключение каналов, комплект документации на семи языках | Committed under Ercan ER's signature |
| **Mihenk** (Claude Opus 5) | Рецензент архитектуры и судья; ADR 0001–0004, блокировка судьи, Билеты A · B · C, проверка полезной нагрузки | Committed under Ercan ER's signature |
| **GitHub Copilot** (Editor Assistant) | Автодополнение кода в редакторе; повторяющиеся паттерны Rust, ветви `match`, леса для тестов (test scaffolding) | No separate git signature |
| **ZAI GLM 5.3** (Zhipu AI) | Поисковая лестница и рекомендации по оптимизации каналов (commit `139b713`) | Committed under Ercan ER's signature |

**Заметка о честности (Honesty note):** Kassam, Mihenk, ZAI GLM 5.3 и GitHub Copilot редактор-ассистент не делают коммиты под отдельными git подписями; их изменения записаны под подписью Ercan ER. Прямые боты GitHub, открывающие PR (`copilot-swe-agent[bot]`: 2, `Devin AI`: 1), появляются со своими собственными подписями.

---

## 📄 7. Лицензия (License)

Этот проект лицензирован под **MIT License** — подробнее в файле `LICENSE`.
Copyright: 2026 Ercan ER.

Документация: `CONTEXT.md` (глоссарий), `docs/adr/` (0001–0004 architecture decisions), `docs/YOL-HARITASI-KAYNAKLAR.md` (генерации источников), `docs/channels/mevzuat.md`, `docs/channels/KANAL-EKLEME.md`, `docs/integration/mcp.md`, `docs/channels/rss.md`.
