**🌍 [Türkçe](README.md) | [English](README.en.md) | [العربية](README.ar.md) | [日本語](README.ja.md) | [中文](README.zh.md) | [Русский](README.ru.md) | [Español](README.es.md)**

# Agent Reach RS (`agent-reach-rs`)

> **一个纯Rust阅读引擎，赋予AI智能体洞察整个互联网的视野**

`agent-reach-rs` 是一个模块化Rust工作空间，让AI智能体（Hermes, Claude, Codex, OpenCode）能够可靠地从网页、社交网络、法律及学术数据库、以及本地音视频文件中读取数据。

本文档基于以下测量结果：`git rev-parse --short HEAD` → `dbe3b0c`，`cargo test --workspace` → **58 passed, 0 failed, 1 ignored**，`harness/gates` → **8 gates green** (7 September 2026)。

---

## 🎯 1. 目标与已完成功能

- **17个频道读取器（channel readers）** — 每个频道尝试一个或多个后端；如果一个失败，则尝试下一个（CLI → API → scraper 阶梯机制）。
- **无外部FFmpeg依赖 (`MediaInspector`)：** 使用 `symphonia` 0.5 解析 MP3, WAV, AAC, FLAC, OGG 和 MKV，无需任何外部 `ffmpeg` 二进制文件。`format_name` 字段始终返回 `symphonia-native`。
- **频道开关 (`disabled_channels`)：** 可以通过配置或环境变量关闭不需要的数据源。禁用的频道会产生**与损坏频道不同的错误** — 前者是决策，后者是需要排查的故障。
- **负载验证 (`require_payload`, ADR 0004)：** 捕获返回 HTTP 200 但没有实际负载内容（如登录墙、下线通知、404落地页）的响应，且不会静默将其视为成功。
- **推送前黄金关卡 (`.githooks/pre-push`)：** 在推送前于本地运行8个质量关卡；如果格式化或构建检查失败，则停止推送。
- **CI 对等关卡 (`gate_ci_parity`)：** 验证 CI 工作流中定义的所有 `cargo` 检查在本地测试集（harness）的关卡中皆已存在。
- **磁带系统（Cassette system）：** 记录并回放响应；失败也会被记录，以便可以在没有网络的情况下执行“未测量”路径。默认关闭。
- **MCP 服务器：** Model Context Protocol 2024-11-05，stdio JSON-RPC，五个工具。
- **本地转录 (`transcribe`)：** 通过 Groq 或 OpenAI 使用 Whisper (`whisper-large-v3-turbo`) 转录本地音视频文件。
- **裁判保护的测量机制：** 黄金集合、验收阈值和防篡改检查存在于单独的文件中；测量端无法篡改被测端。

### Channels (17)

| Channel | Actions | Note |
| :--- | :--- | :--- |
| `web` | `read` | 通过 Jina Reader 读取页面文本 |
| `rss` | `fetch`, `parse` | RSS 2.0 + Atom；Substack 出版物也使用此频道 |
| `twitter` | `search`, `timeline`, `thread` | 两个后端 |
| `youtube` | `metadata`, `transcript`, `search` | 两个后端 |
| `github` | `repo`, `issue`, `pr`, `search` | `gh` CLI + REST API |
| `reddit` | `subreddit`, `search`, `post` | PRAW + REST API |
| `bilibili` | `video`, `info`, `search` | 两个后端 (哔哩哔哩) |
| `xiaohongshu` | `note`, `search` | Web 后端 (小红书) |
| `linkedin` | `profile`, `company`, `search` | 需要凭证 |
| `v2ex` | `topic`, `node`, `hot`, `latest` | Open API |
| `xueqiu` | `quote`, `stock`, `timeline`, `search` | 市场数据 (雪球) |
| `xiaoyuzhou` | `podcast`, `episode` | 播客 (小宇宙) |
| `exa` | `search` | 语义搜索，需要密钥 |
| `duckduckgo` | `search` | HTML 提取器，无需密钥 |
| `turath` | `search`, `book`, `author`, `page` | 经典伊斯兰著作；返回**卷号和页码** |
| `mevzuat` | `mevzuat`, `metin`, `fihrist` | 土耳其立法 (`mevzuat.gov.tr`)，直接根据法律编号查询 |
| `uinjkt` | `identify`, `journals`, `articles`, `record` | UIN Jakarta 伊斯兰学术期刊 (OAI-PMH 2.0, 开放获取) |

### 下一步即将接入的数据源

已在 `docs/YOL-HARITASI-KAYNAKLAR.md` 记录且完成端点测量的有：
**UYAP Emsal** 和 **Yargıtay Karar Arama** (均为无需密钥的 JSON，`aramalist` + `getDokuman`)，**Danıştay** (端点可用，搜索体待解析)，**Pew Research** (`robots.txt` 明确允许 agent 访问并标记 `ai-input=yes`)，**Lexpera** (订阅墙 + 强制 Crawl-delay 10 的人工卡口模型)，以及 Wikipedia, Stanford Encyclopedia of Philosophy, TDV Encyclopedia of Islam, OpenAlex, Crossref，和 Turkish National Thesis Center。

---

## 🏗️ 2. 架构与模块

```text
agent-reach-rs/
├── Cargo.toml                    # 工作空间 (symphonia, tokio, reqwest, clap)
├── .githooks/                    # Rust 原生 git 钩子 (pre-push, post-commit, post-push)
├── crates/
│   ├── agent-reach-core/         # Channel, Backend, Config, Cassette, require_payload,
│   │                             #   MediaInspector, doctor, error
│   ├── agent-reach-channels/     # 17 个频道实现 (包含 mevzuat, uinjkt, turath)
│   ├── agent-reach-mcp/          # MCP JSON-RPC 服务器 (stdio)
│   └── agent-reach-cli/          # 命令行客户端 (二进制文件: agent-reach)
├── harness/                      # 裁判关卡 — 单独构建 (位于工作空间外)
│   ├── kabul.json                # 验收阈值 (与黄金集合隔离的文件)
│   └── biletler/                 # 门票 A · B · C 及其结果
└── docs/
    ├── adr/                      # 架构决策 0001 · 0002 · 0003 · 0004
    └── YOL-HARITASI-KAYNAKLAR.md # 数据源规划以及已测量的端点
```

**为什么 `harness` 位于工作空间外：** 如果运行关卡的工具与受测的代码树被一起编译，代码中的错误会拉着裁判系统一同崩溃。

### 核心类型

| Type | Responsibility |
| :--- | :--- |
| `Channel` | 数据源的名称、动作及执行器 |
| `Backend` | 频道下的一条单一路径；通过 `BackendStatus` 报告健康状况 |
| `Config` | 读写 `~/.agent-reach/config.yaml`，强制实施 `channel_enabled` |
| `Cassette` | 响应记录与回放 (`AGENT_REACH_CASSETTE`) |
| `MediaInspector` | 纯 Rust 媒体解析器 (`symphonia`) |
| `HealthCheck` | 支撑 `doctor` 输出的数据类型 |

---

## 🚀 3. 安装与配置

### 前置条件

- **Rust 1.75+** (`cargo` 和 `rustc`)。
- **无需外部二进制文件：** FFmpeg，Python 和 Node.js 均非强制依赖。缺失可选工具（如作为 GitHub CLI 回退路径使用的 `gh`）会有控制地优雅降级至基于 REST 的替代方案。

### 构建

```bash
git clone https://github.com/Ercaner1988/agent-reach-rs.git
cd agent-reach-rs
cargo build --release
```

生成的二进制文件： `target/release/agent-reach` (Windows 环境为 `agent-reach.exe`)。

> **注意：** 此代码仓不发布随时可下载的预构建二进制文件。`dist-workspace.toml` 将 `installers = []` 设置为空；发布资产仅以压缩包形式产出。从源码构建是唯一的途径。

### 首次配置

```bash
agent-reach install                      # 创建 ~/.agent-reach 目录
agent-reach configure github_token <tk>  # 设置一个密钥
agent-reach configure --json             # 查看当前设置
agent-reach doctor                       # 频道健康检查
agent-reach doctor --json                # 机器可读的健康报告
```

`install` 命令**不会**安装外部工具；它仅准备配置目录。

### 禁用频道

`~/.agent-reach/config.yaml`:

```yaml
disabled_channels:
  - reddit
  - linkedin
```

或者，单次运行中无需修改配置文件：

```bash
AGENT_REACH_DISABLED_CHANNELS=reddit,linkedin agent-reach execute --task-file tasks.json
```

名称比对忽略大小写和空格；不支持前缀匹配（输入 `red` 并不会禁用 `reddit`）。它对 CLI 和 MCP 路径同时生效。

### 识别的配置键

`twitter_auth_token` · `twitter_ct0` · `groq_api_key` · `openai_api_key` · `github_token` · `reddit_client_id` · `reddit_client_secret` · `reddit_user_agent` · `linkedin_username` · `linkedin_password` · `exa_api_key` · `proxy` · `disabled_channels`

---

## 📖 4. 使用与示例

### A. 命令行

`agent-reach --help` 提供的子命令：

```text
install     配置初始化目录（不安装外部工具）
configure   设置配置值或从浏览器进行自动提取
doctor      检查各平台可用性
skill       管理智能体的技能注册
execute     从 JSON 文件中执行任务
transcribe  转录本地音频/视频文件
```

### B. 执行任务

各频道的操作通过一个任务文件驱动（CLI 支持全部 17 个频道）：

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

输出 (`log.json`) 的模式大纲：

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

### C. 本地转录

```bash
agent-reach transcribe recording.mp3
agent-reach transcribe lecture.mkv --provider groq --output lecture.txt
```

只接受本地文件。如果 `groq_api_key` 与 `openai_api_key` 都没被配置，该命令会直观报错并将用户导向 `configure`。

### D. 纯 Rust 媒体解析 (`MediaInspector`)

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

### E. MCP 服务器

`agent-reach-mcp` 二进制文件通过 stdio 进行 JSON-RPC 通信 (协议版本 `2024-11-05`) 并且默认暴露五个工具：

| Tool | Parameters |
| :--- | :--- |
| `web_read` | `url` |
| `rss_fetch` | `url` |
| `rss_parse` | `xml` |
| `exa_search` | `query`, `num_results` (1–10) |
| `agent_reach_execute` | `channel`, `action`, `args[]` |

`agent_reach_execute` 支持这十二个频道：`bilibili`, `duckduckgo`, `github`, `linkedin`, `reddit`, `turath`, `twitter`, `v2ex`, `xiaohongshu`, `xiaoyuzhou`, `xueqiu`, `youtube`。`web`, `rss` 和 `exa` 是通过它们自己的专属工具进行访问的 — 因此有十四个频道在 MCP 下可用。

> **诚实声明 (Honesty note):** 尽管全部 17 个频道都能通过 CLI `execute` 执行，但 `mevzuat` 和 `uinjkt` 尚未添加到 MCP `agent_reach_execute` 的枚举列表或者 `doctor` 命令中；它们计划在下一版本加入。

Hermes / Claude Desktop 配置方法:

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

## 🛡️ 5. 质量关卡与测试

### 已测量的测试状态

```bash
cargo test --workspace
```

对该代码仓在 2026年9月7日 运行产生的真实输出：

| Suite | Result |
| :--- | :--- |
| `agent_reach_channels` (lib) | 39 passed · 0 failed |
| `agent_reach_core` (lib) | 16 passed · 0 failed |
| `search_gauntlet` (integration) | 3 passed · 0 failed · 1 ignored |
| **Total** | **58 passed · 0 failed · 1 ignored** |

那个唯一被忽略的测试 (`ignored`) 依赖真实在线网络；若要测试可使用 `--ignored` 手动运行。

### 裁判系统与 8 个黄金关卡

`harness` 是一个单独的 crate，它通过 `cargo run --manifest-path harness/Cargo.toml -- gates` 运行8个质量关卡：

1. **build** — 验证工作空间的编译。
2. **clippy** — 强制实施零警告 (`-D warnings`)。
3. **unit tests** — 切保所有单元测试必须通过。
4. **formatting** — 检查 `cargo fmt` 格式化规范。
5. **answer-key grep** — 确保没有黄金集合的答案文本被直接固化进源码（总共扫描 138 个短语）。
6. **referee watch** — 验证裁判文件未被擅自修改。
7. **criteria wiring** — 确认每个被配置的验收标准正在主动检查阶段被使用。
8. **ci parity** — 保证所有 CI 检查均完整实现在本地 harness 的关卡设计里。

阈值配置位于 `harness/kabul.json` 中，与黄金集合**分开**（`min_recall_ratio: 0.9`，`max_zero_results: 0`）。而黄金集合包含位于 `crates/agent-reach-channels/tests/golden_search.json` 里的 **24 个案例** (24 cases)。

**公开承认的一个差距 (gap)：** 二十四个案例全部专门指向 `github`。门票 (Ticket) A 的前置条件 — “一个新频道直到在黄金集合中取得至少一场胜利，才视为运作健全” — 对 `turath`，`mevzuat` 以及 `uinjkt` **尚未达成**。即便这些频道已经被通过真实请求做过在线测量，然而其测试案例目前尚未编织进黄金集合中。

### 测量伦理

- 依据 `docs/adr/0003-a-refusal-is-not-a-verdict.md`：传输层的拒绝（例如 HTTP 429、202）**不应被视为平台的能力丢失或漏报 (capability miss)**；它将被标记为 `Not measured` (未测量)。
- 依据 `docs/adr/0004-a-200-is-not-a-payload-verdict.md`：响应了 HTTP 200 然而缺乏真正有效负载数据 (`require_payload`) 的包体会被算作错误处理。

---

## 👥 6. 贡献者

下方提交计数取自 `git shortlog -sne --all` (7 September 2026, 累计共 77 commits)。

| Name / Identity | Role and Contributions | Measurement |
| :--- | :--- | :--- |
| **Ercan ER** ([@Ercaner1988](https://github.com/Ercaner1988)) | 项目负责人与核心架构师；完成 Rust 架构、17-channel 设计、数据源规划、测量伦理 (ADR 0001–0004) | **74 commits** (43 + 22 + 9, 合并三个签名) |
| **copilot-swe-agent[bot]** ([@copilot-swe-agent[bot]](https://github.com/apps/copilot-swe-agent)) | PR #2: 修复了 CI 中的 `cargo fmt` 以及 lint-and-format 的格式化问题 | **2 commits** |
| **Devin AI** ([@devin-ai-integration[bot]](https://github.com/apps/devin-ai-integration)) | PR #1: 提供跨平台 Python 路径嗅探，保护 LinkedIn/Reddit 注入检测以及 CLI 路由设计 | **1 commit** |
| **Kassam** (Hermes agent) | AI 同行开发者；重构 `MediaInspector`，增加 `turath` 频道和频道切换机制，产出七语言文档套件 | 在 Ercan ER 的签名下提交 |
| **Mihenk** (Claude Opus 5) | 架构独立审查与测试裁判者；参与制订 ADR 0001–0004，裁判锁机制，验收门票 A · B · C 系统，与负载验证 | 在 Ercan ER 的签名下提交 |
| **GitHub Copilot** (Editor Assistant) | 位于编辑器中的代码片段协助；模式匹配补全 (`match` arms)、大量 Rust 样板代码与测试脚手架起草 | 无独立的 git 签名 |
| **ZAI GLM 5.3** (Zhipu AI) | 对搜索阶梯和各频道的流程提供优化建议 (参与 commit `139b713`) | 在 Ercan ER 的签名下提交 |

**诚实声明 (Honesty note):** Kassam, Mihenk, ZAI GLM 5.3，以及 GitHub Copilot 编辑助手均没有以各自单独的 git 签名提交；他们的直接变更都登记在了 Ercan ER 本人的签名历史中。直接通过创建 PR 的 GitHub 机械机器人 (`copilot-swe-agent[bot]`: 2, `Devin AI`: 1) 则携带它们自己的签名。

---

## 📄 7. 许可证

本项目依据 **MIT License** 获得许可 —— 详情请查阅 `LICENSE` 协议文件。
Copyright: 2026 Ercan ER。

配套文档: `CONTEXT.md` (词汇表)，`docs/adr/` (0001–0004 架构决策)，`docs/YOL-HARITASI-KAYNAKLAR.md` (数据源世代图谱)，`docs/channels/mevzuat.md`，`docs/channels/KANAL-EKLEME.md`，`docs/integration/mcp.md`，`docs/channels/rss.md`。
