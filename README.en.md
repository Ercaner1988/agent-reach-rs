**🌍 [Türkçe](README.md) | [English](README.en.md) | [العربية](README.ar.md) | [日本語](README.ja.md)**

---

# 👁️ Agent Reach (Rust)

**100% Rust-native internet reading & semantic search layer for AI agents**

> **Note:** This project is a complete Rust rewrite of [Agent Reach](https://github.com/Panniantong/agent-reach) Python version (not an upstream contribution, but a standalone new implementation). Goal: zero Python dependencies, single binary, pure Rust compilation, high performance.

---

## 🎯 Roadmap & Completed Components

### Completed Core Components
- [x] **Workspace skeleton** — 4 crates (`core`, `channels`, `mcp`, `cli`)
- [x] **Web channel** — Jina Reader (`r.jina.ai`) integration
- [x] **RSS channel** — RSS 2.0 and Atom feed fetch & parse (`fetch` + `parse`)
- [x] **Twitter** — `twitter-cli` (authenticated), Nitter (anonymous)
- [x] **YouTube** — `yt-dlp` (metadata, transcript, search)
- [x] **GitHub** — `gh` CLI (relaxation ladder), GitHub REST API
- [x] **Reddit** — Reddit API (OAuth2), PRAW (Python)
- [x] **Chinese Social & Finance** — Bilibili, Xiaohongshu, V2EX, Xueqiu, Xiaoyuzhou
- [x] **Professional & Search** — LinkedIn, Exa Search, DuckDuckGo (HTML search)
- [x] **CLI** — `install`, `configure`, `doctor`, `skill`, `transcribe`, `execute`
- [x] **MCP server** — stdio JSON-RPC, 5 tools (`web_read`, `rss_fetch`, `rss_parse`, `exa_search`, `agent_reach_execute`)
- [x] **Multi-platform build** — Windows, Linux, macOS (`cargo-dist`)
- [x] **CI/CD Pipeline** — GitHub Actions CI/CD & automated test gates (`harness/` (Rust))

Roadmap resources: [`docs/YOL-HARITASI-KAYNAKLAR.md`](docs/YOL-HARITASI-KAYNAKLAR.md)

### Known Limitations (not yet implemented)
- The `agent-reach-graph` (semantic mind map) crate is planned; it does not exist in the repository yet.
- The YouTube `rustube` backend is a placeholder; the working path is the `yt-dlp` subprocess.
- The Twitter Nitter fallback does placeholder-level simple HTML extraction.
- `configure --from-browser` (browser cookie extraction) is not implemented.
- `install` only prepares the configuration directory; it does not install external tools such as `gh`, `yt-dlp`, or `twitter-cli`.
- Reddit requires OAuth2 credentials (`reddit_client_id`, `reddit_client_secret`).

---

## 🏗️ Architecture

```
agent-reach-rs/
├── crates/
│   ├── agent-reach-core/      # Configuration, Backend/Channel traits, Cassette Cache
│   ├── agent-reach-channels/  # 15 platform readers (web, youtube, twitter, github, ...)
│   ├── agent-reach-mcp/       # MCP stdio JSON-RPC server
│   └── agent-reach-cli/       # Clap CLI (install, doctor, skill, execute)
├── harness/                   # Automated audit gates and cassette cache store
└── Cargo.toml                 # Workspace root
```

### Backend Strategy

Each channel defines multiple backends (first choice + fallback):
- **Twitter:** `twitter-cli` $\rightarrow$ fallback: `Nitter`
- **Reddit:** `Reddit API` $\rightarrow$ fallback: `PRAW`
- **YouTube:** `yt-dlp` subprocess (metadata, transcript, search); the `rustube` backend is a placeholder
- **GitHub:** `gh` CLI (unquoted term splitting) $\rightarrow$ fallback: `GitHub REST API`

---

## 🚀 Installation

### Building from Source

```bash
git clone https://github.com/Ercaner1988/agent-reach-rs.git
cd agent-reach-rs
cargo build --release
./target/release/agent-reach --help
```

### Stable Release Installation

```bash
cargo install agent-reach-cli
agent-reach install --env=auto
agent-reach doctor  # Health and dependency check
```

---

## 📖 Usage & Execution

### Web Channel — Single Page Read

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

### Batch Task Execution (`tasks.json`)

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

## 🛡️ Automated Test Gates

Every addition to the project passes through 6 free test gates (`harness/` (Rust)):

```bash
cargo run --manifest-path harness/Cargo.toml -- gates
```

- **Gate 1 (Build):** `cargo build --workspace`
- **Gate 2 (Clippy):** `cargo clippy --workspace --all-targets -- -D warnings`
- **Gate 3 (Unit Tests):** `cargo test --workspace`
- **Gate 4 (Formatting):** `cargo fmt --check`
- **Gate 5 (Cheat Grep):** Automated scan preventing answer key phrases from leaking into source
- **Gate 6 (Gatekeeper):** Git reference validation of referee files

---

## 👥 Contributors

For the complete detailed list, see [`CONTRIBUTORS.md`](CONTRIBUTORS.md).
- **Ercan ER** ([@Ercaner1988](https://github.com/Ercaner1988)) — Project Lead & Architect
- **Kassam** (Hermes Agent / Nous Research) — AI Peer & Co-Developer
- **Mihenk** (Claude Opus 5 / Anthropic) — Peer Reviewer & Referee
- **Devin AI** — Automated Contributor
