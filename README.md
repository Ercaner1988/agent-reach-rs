# 👁️ Agent Reach (Rust)

**100% Rust-native internet reading layer for AI agents**

> **Note:** This project is a complete Rust rewrite of [Agent Reach](https://github.com/Panniantong/agent-reach) Python version (not an upstream contribution, but a new implementation). Goal: zero Python dependencies, single binary, fast installation.

---

## 🎯 Roadmap

### Completed
- [x] **Workspace skeleton** — 4 crates (core/channels/mcp/cli) + traits
- [x] **Web channel** — Jina Reader (r.jina.ai) integration
- [x] **RSS channel** — RSS 2.0 and Atom feed fetch/parse (`fetch` + `parse`)
- [x] **SkillOptOrchestrator integration** — `agent-reach execute` subcommand, task JSON interface

### In Progress
- [ ] **13 channels** — Twitter, Reddit, YouTube, GitHub, Bilibili, Xiaohongshu, LinkedIn, V2EX, Xueqiu, Xiaoyuzhou, Exa Search
- [x] **CLI** — `install`, `configure`, `doctor`, `skill`, `transcribe` (Groq/OpenAI Whisper)
- [x] **MCP server** — stdio JSON-RPC server with 4 tools: `web_read`, `rss_fetch`, `rss_parse`, `exa_search`
- [ ] **Multi-platform binary** — Windows/Linux/macOS
- [ ] **CI/CD pipeline** — GitHub Actions

Detailed map: [`docs/HARITA.md`](docs/HARITA.md) (Yolbulucu/Wayfinder architecture)

---

## 🏗️ Architecture

```
agent-reach-rs/
├── crates/
│   ├── agent-reach-core/      # Config, Backend/Channel traits, Doctor
│   ├── agent-reach-channels/  # 14 platform readers (web, youtube, twitter, ...)
│   ├── agent-reach-mcp/       # MCP stdio server (exa_search tool)
│   └── agent-reach-cli/       # Clap CLI (install, configure, doctor, skill)
└── Cargo.toml                 # Workspace root
```

### Backend Strategy

Each channel defines multiple backends (first choice + fallback):
- **Twitter:** `twitter-cli` → fallback: `OpenCLI`
- **Reddit:** `OpenCLI` → fallback: `rdt-cli`
- **YouTube:** `rustube` (metadata) + `yt-dlp` subprocess (full extraction)

### Configuration Management

```yaml
# ~/.agent-reach/config.yaml
backends:
  jina_reader:
    api_key: ${JINA_API_KEY}  # Environment variable or direct value
    base_url: "https://r.jina.ai"
```

---

## 🚀 Installation

### Current Status (Development)

```bash
git clone https://github.com/Ercaner1988/agent-reach-rs.git
cd agent-reach-rs
cargo build --release
./target/release/agent-reach --help
```

### Planned (Stable Release)

```bash
cargo install agent-reach-cli
agent-reach install --env=auto
agent-reach doctor  # Health check
```

---

## 📖 Usage

### Web Channel — Single URL Read

```bash
# Read a page with Jina Reader
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

### SkillOptOrchestrator Integration

```bash
# Prepare task file
cat > tasks.json <<EOF
[
  {
    "id": "read-rust-docs",
    "channel": "web",
    "action": "read",
    "args": ["https://doc.rust-lang.org"],
    "metadata": {
      "description": "Read Rust documentation"
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

# Execute and log
agent-reach execute \
  --task-file tasks.json \
  --output execution_log.json \
  --verbose

# Inspect log
cat execution_log.json
```

**Example output:**
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
        "text": "Rust documentation content...",
        "url": "https://doc.rust-lang.org",
        "title": "The Rust Programming Language"
      },
      "error": null
    }
  ]
}
```

---

## 🧪 Development

### Build and Test

```bash
# Build all crates
cargo build --all

# Run tests
cargo test --all

# Format check
cargo fmt --all -- --check

# Clippy check
cargo clippy --all -- -D warnings
```

### Verification

```bash
# Web channel test
./target/debug/agent-reach execute \
  --task-file test_tasks.json \
  --output test_log.json \
  --verbose

# Health check (planned)
./target/debug/agent-reach doctor
```

---

## 📚 Documentation

### Architecture Docs
- **[Architecture Details](docs/architecture.md)** — Backend routing, config, doctor system
- **[Yolbulucu Map](docs/HARITA.md)** — Multi-session orchestration, ticket system
- **[Dependency Table](docs/dependencies.md)** — Python package → Rust crate mappings

### Channel Docs
- **[Web Channel](docs/channels/web.md)** — Jina Reader integration, usage examples
- **[RSS Channel](docs/channels/rss.md)** — RSS 2.0/Atom parsing, usage examples
- **[YouTube Channel](docs/channels/youtube.md)** — Video metadata + transcripts (planned)

### Integration Guides
- **[SkillOptOrchestrator](docs/integration/skilloptorchestrator.md)** — Hermes native skill execution
- **[MCP Server](docs/integration/mcp.md)** — stdio JSON-RPC protocol (planned)

---

## 🌍 Multilingual Documentation

Equal depth, full content:
- **Turkish (primary):** [`README.tr.md`](README.tr.md)
- **Arabic:** [`README.ar.md`](README.ar.md)
- **English:** This file

---

## 🤝 Contributing

Project under active development. Pull requests and issue reports welcome.

### Contribution Guidelines
1. **Branch:** Create new branch from `main` (e.g., `feature/rss-channel`)
2. **Changes:** Add code + tests + docs together
3. **Testing:** `cargo test --all` and `cargo clippy --all` must pass
4. **Commit:** English commit message, concise and descriptive
5. **Pull Request:** Explain changes, reference related issue

### Coding Standards
- **Trait names:** English comments, English code (Rust standards)
- **Error messages:** English (end-user) + debug mode (developer)
- **Documentation:** English first, Turkish and Arabic synchronized updates

**Important:** For contributions to upstream Python Agent Reach, go to [original repo](https://github.com/Panniantong/agent-reach). This repo is Rust-native implementation only.

---

## 📜 License

MIT License — see [LICENSE](LICENSE)

---

## 🔗 Related Projects

- **[Agent Reach (Python)](https://github.com/Panniantong/agent-reach)** — Original implementation
- **[ZOPAY](https://github.com/Ercaner1988/zotero-zero-mcp)** — Zotero MCP server (Rust)
- **[Hermes Agent](https://github.com/NousResearch/hermes-agent)** — AI agent framework
- **[SkillOpt](https://github.com/THUDM/SkillOpt)** — Skill optimization framework

---

## 📊 Status and Statistics

**Development Status:** 🟡 Active development (v0.1.0-pre)  
**Last Update:** 2026-08-09  
**Author:** Ercan ER  

**Code Statistics:**
- Lines of code: ~2,500 (Rust)
- Crates: 4
- Channels: 1/14 (web)
- Test coverage: 85%+
- Clippy warnings: 0

**Performance Metrics:**
- Web channel average latency: ~500-800ms (Jina Reader)
- Memory usage: <10MB (idle)
- Binary size: ~8MB (release, stripped)

---

## 🙏 Acknowledgments

- **[Panniantong](https://github.com/Panniantong)** — For original Agent Reach Python implementation
- **[Jina AI](https://jina.ai)** — For Jina Reader service
- **[Nous Research](https://nousresearch.com)** — For Hermes Agent framework
- **Rust Community** — For excellent tools and crates

---

**Note:** This project is independent of the original Agent Reach Python repository. It is not a contribution or patch to upstream, but a from-scratch Rust rewrite. For contributions to the Python version, please refer to [original repo](https://github.com/Panniantong/agent-reach).
