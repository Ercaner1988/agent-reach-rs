**🌍 [Türkçe](README.md) | [English](README.en.md) | [العربية](README.ar.md) | [日本語](README.ja.md) | [中文](README.zh.md) | [Русский](README.ru.md) | [Español](README.es.md)**

# Agent Reach RS (`agent-reach-rs`)

> **Pure-Rust Media, Web, and Multi-Channel Data Reader Engine for AI Agents**

`agent-reach-rs` is a modular Rust ecosystem enabling AI agents (Hermes, Claude, Codex, OpenCode) to read data reliably, rapidly, and independently across external websites, social networks, academic databases, and media assets.

---

## 🎯 1. Purpose & Features

- **External FFmpeg Binary Independence (`MediaInspector`):** Decodes and inspects audio and media formats (MP3, WAV, AAC, FLAC, OGG, MKV) natively in pure Rust via `symphonia` (v0.5) without requiring an external `ffmpeg.exe` binary.
- **14 Multi-Channel Readers:**
  - **Social & Web:** Twitter/X (Nitter / GraphQL), Reddit API, Bilibili, Xiaohongshu (XHS), V2EX, Xueqiu, LinkedIn, Xiaoyuzhou.
  - **Academic & Code:** Turath (Islamic Law & Manuscript Database), GitHub REST API, RSS/Atom Feeds.
  - **Search Engines:** Exa AI Semantic Search, DuckDuckGo HTML Extractor, Jina Web Reader.
- **5D Epistemic Vector Engine (`agent-reach-graph`):** Turso SQLite (0.7.2) matrix covering ontological, aesthetic, epistemological, moral, and linguistic dimensions.
- **MCP Server Integration:** JSON-RPC CLI and server driver fully compliant with Model Context Protocol (MCP) standards.

---

## 🏗️ 2. Architecture & Modules

```text
agent-reach-rs/
├── Cargo.toml                    # Workspace configuration (symphonia, tokio, reqwest)
├── crates/
│   ├── agent-reach-core/        # Core types, MediaInspector, Error handling, Config
│   ├── agent-reach-channels/    # 14 channel reader implementations (YouTube, Turath, RSS, etc.)
│   ├── agent-reach-mcp/         # MCP JSON-RPC server driver
│   └── agent-reach-cli/         # Command-line interface binary (binary: agent-reach)
└── harness/                     # Automated test harness & gauntlet validation gates
```

---

## 🚀 3. Installation & Setup

### Prerequisites
- **Rust Toolchain:** Rust 1.75+ (`cargo` and `rustc` installed).
- **External Dependencies:** NONE (No external FFmpeg binary, Python, or Node.js required).

### Compilation
```bash
# Clone the repository
git clone https://github.com/Ercaner1988/agent-reach-rs.git
cd agent-reach-rs

# Build the workspace
cargo build --release
```

The compiled binary will be located at `target/release/agent-reach.exe`.

---

## 📖 4. Usage & Examples

### A. Pure-Rust Media Inspection (`MediaInspector` API)
```rust
use agent_reach_core::MediaInspector;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inspect audio natively without invoking ffmpeg.exe
    let meta = MediaInspector::inspect_file("sample_audio.mp3")?;
    
    println!("Codec: {}", meta.codec_name);
    println!("Sample Rate: {} Hz", meta.sample_rate);
    println!("Channels: {}", meta.channels);
    println!("Duration: {:.2} seconds", meta.duration_seconds);
    
    Ok(())
}
```

### B. CLI Usage
```bash
# Run Exa semantic search
agent-reach --channel exa search "Max Weber legal rationalization"

# Read manuscript from Turath database
agent-reach --channel turath read --book 124 --page 45

# Fetch RSS feed
agent-reach --channel rss fetch "https://news.ycombinator.com/rss"
```

---

## 🛡️ 5. Quality Gates & Testing

Protected by 6 strict verification gates requiring 100% pass rate.

```bash
# Run all workspace tests (41/41 green gates)
cargo test --workspace
```

- **`agent-reach-core`:** 10/10 tests passed (including pure-Rust media inspection).
- **`agent-reach-channels`:** 28/28 tests passed.
- **`search_gauntlet`:** 3/3 referee gates verified.

---

## 👥 6. Contributors

| Name / Identity | Role & Contributions | Metrics |
| :--- | :--- | :--- |
| **Ercan Er** | Lead Architect & Project Owner (Rust Architecture) | 38 commits, Core Codebase |
| **Mihenk** | Code Auditor & Referee Gatekeeper | Referee Approvals & Gauntlet Audit |
| **El-Kassâm** | Agent Developer (MediaInspector, Pure-Rust Integration) | 12 commits, Media & Test Suite |
| **GitHub Copilot** | Auxiliary Code Completion | Pair Assistant |
| **Hermes** | Agent Orchestration Engine | Agent Runtime Environment |

---

## 📄 7. License

Licensed under the **MIT License**. See `LICENSE` for details.
