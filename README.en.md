**🌍 [Türkçe](README.md) | [English](README.en.md) | [العربية](README.ar.md) | [日本語](README.ja.md) | [中文](README.zh.md) | [Русский](README.ru.md) | [Español](README.es.md)**

# Agent Reach RS (`agent-reach-rs`)

> **A pure-Rust reading engine that gives AI agents eyes to see the entire internet**

`agent-reach-rs` is a modular Rust workspace that lets AI agents (Hermes, Claude, Codex, OpenCode) reliably read data from web pages, social networks, legislation and academic databases, and local audio/video files.

Measurements this document rests on: `git rev-parse --short HEAD` → `dbe3b0c`, `cargo test --workspace` → **58 passed, 0 failed, 1 ignored**, `harness/gates` → **8 gates green** (7 September 2026).

---

## 🎯 1. Goals and Completed Features

- **17 channel readers** — each channel tries one or more backends; if one fails, the next is attempted (CLI → API → scraper ladder).
- **No external FFmpeg dependency (`MediaInspector`):** MP3, WAV, AAC, FLAC, OGG, and MKV are parsed with `symphonia` 0.5 without any external `ffmpeg` binary. The `format_name` field always returns `symphonia-native`.
- **Channel switch (`disabled_channels`):** Unwanted sources can be turned off via config or an environment variable. A disabled channel produces a **different error than a broken channel** — one is a decision, the other is a fault to chase.
- **Payload verification (`require_payload`, ADR 0004):** Responses returning HTTP 200 without actual payload content (login walls, offline notices, 404 landing pages) are caught and not silently treated as successful.
- **Pre-push golden gate (`.githooks/pre-push`):** Runs the 8 quality gates locally before push; halts the push if formatting or build checks fail.
- **CI parity gate (`gate_ci_parity`):** Verifies that all `cargo` checks defined in CI workflows are present in local harness test gates.
- **Cassette system:** Responses are recorded and replayed; failures are recorded too, so the "not measured" path can be exercised without a network. Off by default.
- **MCP server:** Model Context Protocol 2024-11-05, stdio JSON-RPC, five tools.
- **Local transcription (`transcribe`):** A local audio/video file is transcribed with Whisper (`whisper-large-v3-turbo`) through Groq or OpenAI.
- **Referee-protected measurement:** The golden set, acceptance thresholds, and tamper checks live in separate files; the measuring side cannot alter the measured.

### Channels (17)

| Channel | Actions | Note |
| :--- | :--- | :--- |
| `web` | `read` | Page text through Jina Reader |
| `rss` | `fetch`, `parse` | RSS 2.0 + Atom; Substack publications also use this channel |
| `twitter` | `search`, `timeline`, `thread` | Two backends |
| `youtube` | `metadata`, `transcript`, `search` | Two backends |
| `github` | `repo`, `issue`, `pr`, `search` | `gh` CLI + REST API |
| `reddit` | `subreddit`, `search`, `post` | PRAW + REST API |
| `bilibili` | `video`, `info`, `search` | Two backends |
| `xiaohongshu` | `note`, `search` | Web backend |
| `linkedin` | `profile`, `company`, `search` | Requires credentials |
| `v2ex` | `topic`, `node`, `hot`, `latest` | Open API |
| `xueqiu` | `quote`, `stock`, `timeline`, `search` | Market data |
| `xiaoyuzhou` | `podcast`, `episode` | Podcasts |
| `exa` | `search` | Semantic search, requires a key |
| `duckduckgo` | `search` | HTML extractor, no key |
| `turath` | `search`, `book`, `author`, `page` | Classical Islamic works; returns **volume and page numbers** |
| `mevzuat` | `mevzuat`, `metin`, `fihrist` | Turkish legislation (`mevzuat.gov.tr`), queried directly by law number |
| `uinjkt` | `identify`, `journals`, `articles`, `record` | UIN Jakarta Islamic academic journals (OAI-PMH 2.0, open access) |

### Sources queued next

Documented with measured endpoints in `docs/YOL-HARITASI-KAYNAKLAR.md`:
**UYAP Emsal** and **Yargıtay Karar Arama** (both keyless JSON, `aramalist` + `getDokuman`), **Danıştay** (endpoint alive, search body to be solved), **Pew Research** (`robots.txt` explicitly grants agent access, `ai-input=yes`), **Lexpera** (subscription wall + mandatory 10-second crawl delay → human-gate model), plus Wikipedia, Stanford Encyclopedia of Philosophy, TDV Encyclopedia of Islam, OpenAlex, Crossref, and the Turkish National Thesis Center.

---

## 🏗️ 2. Architecture and Modules

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

**Why `harness` sits outside the workspace:** if the tool that runs the gates were compiled together with the tree it measures, a break in the code would take the referee down with it.

### Core types

| Type | Responsibility |
| :--- | :--- |
| `Channel` | A source's name, actions, and executor |
| `Backend` | A single path beneath a channel; reports health via `BackendStatus` |
| `Config` | Reads/writes `~/.agent-reach/config.yaml`, enforces `channel_enabled` |
| `Cassette` | Response recording and replay (`AGENT_REACH_CASSETTE`) |
| `MediaInspector` | Pure-Rust media parser (`symphonia`) |
| `HealthCheck` | Data type behind `doctor` output |

---

## 🚀 3. Installation and Configuration

### Prerequisites

- **Rust 1.75+** (`cargo` and `rustc`).
- **No external binaries required:** FFmpeg, Python, and Node.js are not mandatory. Optional backends for certain channels (`gh` CLI, `praw`, `BBDown`) are used when installed; otherwise the channel falls through to its other backend.

### Building

```bash
git clone https://github.com/Ercaner1988/agent-reach-rs.git
cd agent-reach-rs
cargo build --release
```

Produced binary: `target/release/agent-reach` (`agent-reach.exe` on Windows).

> **Note:** This repository does not publish prebuilt downloadable binaries. `dist-workspace.toml` sets `installers = []`; release assets are produced as archives only. Building from source is the only path.

### First-time configuration

```bash
agent-reach install                      # Creates the ~/.agent-reach directory
agent-reach configure github_token <tk>  # Set a key
agent-reach configure --json             # View current settings
agent-reach doctor                       # Channel health check
agent-reach doctor --json                # Machine-readable health report
```

The `install` command does **not** install external tools; it only prepares the configuration directory.

### Disabling channels

`~/.agent-reach/config.yaml`:

```yaml
disabled_channels:
  - reddit
  - linkedin
```

Or, for a single run without touching the file:

```bash
AGENT_REACH_DISABLED_CHANNELS=reddit,linkedin agent-reach execute --task-file tasks.json
```

Name comparison ignores case and whitespace; there is no prefix matching (writing `red` will not disable `reddit`). It applies to both the CLI and MCP paths.

### Recognised configuration keys

`twitter_auth_token` · `twitter_ct0` · `groq_api_key` · `openai_api_key` · `github_token` · `reddit_client_id` · `reddit_client_secret` · `reddit_user_agent` · `linkedin_username` · `linkedin_password` · `exa_api_key` · `proxy` · `disabled_channels`

---

## 📖 4. Usage and Examples

### A. Command line

Subcommands from `agent-reach --help`:

```text
install     Set up the config directory (external tools are not installed)
configure   Set a config value or auto-extract from browser
doctor      Check platform availability
skill       Manage agent skill registration
execute     Execute tasks from a JSON file
transcribe  Transcribe a local audio/video file
```

### B. Executing tasks

Channels are driven through a task file (all 17 channels supported via CLI):

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

Output (`log.json`) schema:

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

### C. Local transcription

```bash
agent-reach transcribe recording.mp3
agent-reach transcribe lecture.mkv --provider groq --output lecture.txt
```

Only local files are accepted. If neither `groq_api_key` nor `openai_api_key` is configured, the command says so plainly and points to `configure`.

### D. Pure-Rust media parsing (`MediaInspector`)

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

### E. MCP server

The `agent-reach-mcp` binary speaks JSON-RPC over stdio (protocol `2024-11-05`) and exposes five tools:

| Tool | Parameters |
| :--- | :--- |
| `web_read` | `url` |
| `rss_fetch` | `url` |
| `rss_parse` | `xml` |
| `exa_search` | `query`, `num_results` (1–10) |
| `agent_reach_execute` | `channel`, `action`, `args[]` |

`agent_reach_execute` accepts these twelve channels: `bilibili`, `duckduckgo`, `github`, `linkedin`, `reddit`, `turath`, `twitter`, `v2ex`, `xiaohongshu`, `xiaoyuzhou`, `xueqiu`, `youtube`. `web`, `rss`, and `exa` are reached through their own tools — so fourteen channels are available over MCP.

> **Honesty note:** While all 17 channels run through CLI `execute`, `mevzuat` and `uinjkt` have not yet been added to the MCP `agent_reach_execute` enum or the `doctor` command; they are scheduled for the next release.

Hermes / Claude Desktop configuration:

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

## 🛡️ 5. Quality Gates and Tests

### Measured test status

```bash
cargo test --workspace
```

Run against this repository on 7 September 2026, actual output:

| Suite | Result |
| :--- | :--- |
| `agent_reach_channels` (lib) | 39 passed · 0 failed |
| `agent_reach_core` (lib) | 16 passed · 0 failed |
| `search_gauntlet` (integration) | 3 passed · 0 failed · 1 ignored |
| **Total** | **58 passed · 0 failed · 1 ignored** |

The single ignored test requires a live network; run it manually with `--ignored`.

### Referee system and 8 golden gates

`harness` is a separate crate that runs 8 gates via `cargo run --manifest-path harness/Cargo.toml -- gates`:

1. **build** — Workspace compilation validation.
2. **clippy** — Zero-warning enforcement (`-D warnings`).
3. **unit tests** — All unit tests passing.
4. **formatting** — `cargo fmt` formatting compliance.
5. **answer-key grep** — Ensures no golden-set answer strings are copied into source code (138 phrases scanned).
6. **referee watch** — Verifies referee files are unmodified.
7. **criteria wiring** — Confirms acceptance criteria are actively checked.
8. **ci parity** — Guarantees all CI checks exist in local harness gates.

Thresholds live in `harness/kabul.json`, **separate** from the golden set (`min_recall_ratio: 0.9`, `max_zero_results: 0`). The golden set holds **24 cases** in `crates/agent-reach-channels/tests/golden_search.json`.

**A gap recorded openly:** all twenty-four cases target `github`. Ticket A's condition — "a new channel does not count as working until it wins at least one case in the golden set" — is **not yet met** for `turath`, `mevzuat`, and `uinjkt`. The channels were measured live, but they have not entered the golden set.

### Measurement ethics

- Per `docs/adr/0003-a-refusal-is-not-a-verdict.md`: a transport-layer refusal (HTTP 429, 202) is **not counted as a capability miss**; it is marked as `Not measured`.
- Per `docs/adr/0004-a-200-is-not-a-payload-verdict.md`: HTTP 200 responses with no real data (`require_payload`) are treated as errors.

---

## 👥 6. Contributors

The counts below come from `git shortlog -sne --all` (7 September 2026, 77 commits in total).

| Name / Identity | Role and Contributions | Measurement |
| :--- | :--- | :--- |
| **Ercan ER** ([@Ercaner1988](https://github.com/Ercaner1988)) | Project owner and lead architect; Rust architecture, 17-channel design, source generations, measurement ethics (ADR 0001–0004) | **74 commits** (43 + 22 + 9, three signatures merged) |
| **copilot-swe-agent[bot]** ([@copilot-swe-agent[bot]](https://github.com/apps/copilot-swe-agent)) | PR #2: CI `cargo fmt` and lint-and-format formatting fixes | **2 commits** |
| **Devin AI** ([@devin-ai-integration[bot]](https://github.com/apps/devin-ai-integration)) | PR #1: cross-platform Python path detection, LinkedIn/Reddit injection safeguards, CLI routing | **1 commit** |
| **Kassam** (Hermes agent) | AI peer developer; `MediaInspector`, `turath` channel, channel switch, the seven-language documentation suite | Committed under Ercan ER's signature |
| **Mihenk** (Claude Opus 5) | Architecture reviewer and referee; ADR 0001–0004, referee lock, Tickets A · B · C, payload verification | Committed under Ercan ER's signature |
| **GitHub Copilot** (Editor Assistant) | In-editor code completion; repetitive Rust patterns, `match` arms, test scaffolding | No separate git signature |
| **ZAI GLM 5.3** (Zhipu AI) | Search ladder and channel optimization suggestions (commit `139b713`) | Committed under Ercan ER's signature |

**Honesty note:** Kassam, Mihenk, ZAI GLM 5.3, and the GitHub Copilot editor assistant do not commit under separate git signatures; their changes are recorded under Ercan ER's signature. Direct GitHub Bots opening PRs (`copilot-swe-agent[bot]`: 2, `Devin AI`: 1) appear with their own signatures.

---

## 📄 7. License

This project is licensed under the **MIT License** — see the `LICENSE` file.
Copyright: 2026 Ercan ER.

Documentation: `CONTEXT.md` (glossary), `docs/adr/` (0001–0004 architecture decisions), `docs/YOL-HARITASI-KAYNAKLAR.md` (source generations), `docs/channels/mevzuat.md`, `docs/channels/KANAL-EKLEME.md`, `docs/integration/mcp.md`, `docs/channels/rss.md`.
