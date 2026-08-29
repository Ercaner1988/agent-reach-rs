**🌍 [Türkçe](README.md) | [English](README.en.md) | [العربية](README.ar.md) | [日本語](README.ja.md) | [中文](README.zh.md) | [Русский](README.ru.md) | [Español](README.es.md)**

# Agent Reach RS (`agent-reach-rs`)

> **面向 AI Agent 的纯 Rust 媒体、Web 与多通道数据读取引擎**

`agent-reach-rs` 是一个模块化的 Rust 生态系统，使 AI Agent（Hermes、Claude、Codex、OpenCode）能够快速、可靠且独立地从外部网站、社交网络、学术数据库及媒体文件中提取数据。

---

## 🎯 1. 目标与特性

- **脱离外部 FFmpeg 二进制依赖 (`MediaInspector`)：** 无需外部 `ffmpeg.exe` 执行文件，通过 `symphonia` (v0.5) 库直接以纯 Rust 原生解析 MP3、WAV、AAC、FLAC、OGG 及 MKV 等音视频格式。
- **14 个多通道读取器：**
  - **社交与 Web：** Twitter/X (Nitter / GraphQL)、Reddit API、Bilibili、小红书 (XHS)、V2EX、雪球、LinkedIn、小宇宙。
  - **学术与代码：** Turath（伊斯兰法典与手稿数据库）、GitHub REST API、RSS/Atom 订阅。
  - **搜索引擎：** Exa AI 语义搜索、DuckDuckGo HTML 提取器、Jina Web Reader。
- **5D 认识论向量引擎 (`agent-reach-graph`)：** 基于 Turso SQLite (0.7.2) 的本体论、美学、认识论、道德与语言维度矩阵。
- **MCP 服务端集成：** 完全符合 Model Context Protocol (MCP) 标准的 JSON-RPC CLI 与服务端驱动。

---

## 🏗️ 2. 架构与模块

```text
agent-reach-rs/
├── Cargo.toml                    # Workspace 配置 (symphonia, tokio, reqwest)
├── crates/
│   ├── agent-reach-core/        # 核心类型、MediaInspector、错误处理、配置
│   ├── agent-reach-channels/    # 14 个通道读取器实现 (YouTube, Turath, RSS 等)
│   ├── agent-reach-mcp/         # MCP JSON-RPC 服务端驱动
│   └── agent-reach-cli/         # 命令行客户端 (二进制文件: agent-reach)
└── harness/                     # 自动化测试与评审验证关卡
```

---

## 🚀 3. 安装与配置

### 前置要求
- **Rust 工具链：** Rust 1.75+（需安装 `cargo` 与 `rustc`）。
- **外部依赖：** 无（无需外部 FFmpeg 二进制、Python 或 Node.js）。

### 编译
```bash
# 克隆仓库
git clone https://github.com/Ercaner1988/agent-reach-rs.git
cd agent-reach-rs

# 编译 Workspace
cargo build --release
```

编译后的二进制文件位于 `target/release/agent-reach.exe`。

---

## 📖 4. 使用方法与示例

### A. 纯 Rust 媒体解析 (`MediaInspector` API)
```rust
use agent_reach_core::MediaInspector;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 无需调用 ffmpeg.exe，直接原生解析音频文件
    let meta = MediaInspector::inspect_file("sample_audio.mp3")?;
    
    println!("编解码器: {}", meta.codec_name);
    println!("采样率: {} Hz", meta.sample_rate);
    println!("声道数: {}", meta.channels);
    println!("时长: {:.2} 秒", meta.duration_seconds);
    
    Ok(())
}
```

### B. CLI 使用
```bash
# 运行 Exa 语义搜索
agent-reach --channel exa search "Max Weber legal rationalization"

# 从 Turath 数据库读取手稿
agent-reach --channel turath read --book 124 --page 45

# 获取 RSS 订阅
agent-reach --channel rss fetch "https://news.ycombinator.com/rss"
```

---

## 🛡️ 5. 质量关卡与测试

受 6 个严格的验证关卡保护，要求 100% 通过率。

```bash
# 运行 Workspace 所有测试 (41/41 绿色关卡)
cargo test --workspace
```

- **`agent-reach-core`：** 10/10 测试通过（包含纯 Rust 媒体解析）。
- **`agent-reach-channels`：** 28/28 测试通过。
- **`search_gauntlet`：** 3/3 评审关卡验证通过。

---

## 👥 6. 贡献者

| 姓名 / 身份 | 角色与贡献 | 统计指标 |
| :--- | :--- | :--- |
| **Ercan Er** | 首席架构师兼项目所有者 (Rust 架构) | 38 次提交，核心代码库 |
| **Mihenk** | 代码审计员与评审关卡守护者 | 评审批准与 Gauntlet 审计 |
| **El-Kassâm** | Agent 开发者 (MediaInspector，纯 Rust 集成) | 12 次提交，媒体与测试套件 |
| **ZAI GLM 5.3** | Agent 模型贡献与代码编辑 | 模型推理 |
| **GitHub Copilot** | 辅助代码补全 | 配对助手 |
| **Hermes** | Agent 编排引擎 | Agent 运行环境 |

---

## 📄 7. 许可证

本项目基于 **MIT 许可证** 开源。详见 `LICENSE` 文件。
