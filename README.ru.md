**🌍 [Türkçe](README.md) | [English](README.en.md) | [العربية](README.ar.md) | [日本語](README.ja.md) | [中文](README.zh.md) | [Русский](README.ru.md) | [Español](README.es.md)**

# Agent Reach RS (`agent-reach-rs`)

> **Движок чтения медиа, веб-данных и многоканальных источников на чистом Rust для AI-агентов**

`agent-reach-rs` — это модульная экосистема на Rust, позволяющая AI-агентам (Hermes, Claude, Codex, OpenCode) надежно, быстро и независимо извлекать данные с внешних веб-сайтов, социальных сетей, академических баз данных и медиафайлов.

---

## 🎯 1. Назначение и возможности

- **Независимость от внешнего бинарника FFmpeg (`MediaInspector`):** Декодирование и анализ аудио- и медиаформатов (MP3, WAV, AAC, FLAC, OGG, MKV) напрямую на чистом Rust с помощью библиотеки `symphonia` (v0.5) без необходимости использования `ffmpeg.exe`.
- **14 многоканальных ридеров:**
  - **Соцсети и Веб:** Twitter/X (Nitter / GraphQL), Reddit API, Bilibili, Xiaohongshu (XHS), V2EX, Xueqiu, LinkedIn, Xiaoyuzhou.
  - **Академические данные и код:** Turath (база данных исламского права и рукописей), GitHub REST API, ленты RSS/Atom.
  - **Поисковые системы:** Семантический поиск Exa AI, HTML-экстрактор DuckDuckGo, Jina Web Reader.
- **5D эпистемический векторный движок (`agent-reach-graph`):** Матрица онтологических, эстетических, эпистемологических, моральных и лингвистических измерений на базе Turso SQLite (0.7.2).
- **Интеграция с MCP-сервером:** Драйвер JSON-RPC CLI и сервера, полностью соответствующий стандартам Model Context Protocol (MCP).

---

## 🏗️ 2. Архитектура и модули

```text
agent-reach-rs/
├── Cargo.toml                    # Конфигурация Workspace (symphonia, tokio, reqwest)
├── crates/
│   ├── agent-reach-core/        # Базовые типы, MediaInspector, обработка ошибок, Config
│   ├── agent-reach-channels/    # Реализация 14 ридеров каналов (YouTube, Turath, RSS и др.)
│   ├── agent-reach-mcp/         # Драйвер MCP JSON-RPC сервера
│   └── agent-reach-cli/         # Бинарный CLI-клиент (binary: agent-reach)
└── harness/                     # Автоматический тестовый арбитраж и ворота качества
```

---

## 🚀 3. Установка и настройка

### Требования
- **Rust Toolchain:** Rust 1.75+ (установленные `cargo` и `rustc`).
- **Внешние зависимости:** ОТСУТСТВУЮТ (Не требуется FFmpeg, Python или Node.js).

### Сборка
```bash
# Клонирование репозитория
git clone https://github.com/Ercaner1988/agent-reach-rs.git
cd agent-reach-rs

# Сборка workspace
cargo build --release
```

Скомпилированный бинарник будет находиться по адресу `target/release/agent-reach.exe`.

---

## 📖 4. Использование и примеры

### A. Анализ медиа на чистом Rust (`MediaInspector` API)
```rust
use agent_reach_core::MediaInspector;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Анализ аудиофайла без вызова ffmpeg.exe
    let meta = MediaInspector::inspect_file("sample_audio.mp3")?;
    
    println!("Кодек: {}", meta.codec_name);
    println!("Частота дискретизации: {} Hz", meta.sample_rate);
    println!("Каналы: {}", meta.channels);
    println!("Длительность: {:.2} сек", meta.duration_seconds);
    
    Ok(())
}
```

### B. Использование CLI
```bash
# Запуск семантического поиска Exa
agent-reach --channel exa search "Max Weber legal rationalization"

# Чтение рукописи из базы данных Turath
agent-reach --channel turath read --book 124 --page 45

# Получение RSS-ленты
agent-reach --channel rss fetch "https://news.ycombinator.com/rss"
```

---

## 🛡️ 5. Ворота качества и тестирование

Защищено 6 строгими воротами проверки с требованием 100% прохождения.

```bash
# Запуск всех тестов workspace (41/41 зеленых ворот)
cargo test --workspace
```

- **`agent-reach-core`:** 10/10 тестов пройдено (включая анализ медиа на чистом Rust).
- **`agent-reach-channels`:** 28/28 тестов пройдено.
- **`search_gauntlet`:** 3/3 ворот арбитража подтверждены.

---

## 👥 6. Участники проекта

| Имя / Идентификатор | Роль и вклад | Метрики |
| :--- | :--- | :--- |
| **Ercan Er** | Главный архитектор и владелец проекта (Архитектура Rust) | 38 комитов, основная кодовая база |
| **Mihenk** | Аудитор кода и хранитель ворот арбитража | Подтверждения арбитража и аудит Gauntlet |
| **El-Kassâm** | Разработчик агента (MediaInspector, интеграция чистый Rust) | 12 комитов, медиа и набор тестов |
| **ZAI GLM 5.3** | Вклад модели агента и редактирование кода | Рассуждения модели |
| **GitHub Copilot** | Вспомогательное автодополнение кода | Парный ассистент |
| **Hermes** | Движок оркестрации агентов | Среда выполнения агентов |

---

## 📄 7. Лицензия

Распространяется под лицензией **MIT License**. Подробности см. в файле `LICENSE`.
