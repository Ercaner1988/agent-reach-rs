**🌍 [Türkçe](README.md) | [English](README.en.md) | [العربية](README.ar.md) | [日本語](README.ja.md) | [中文](README.zh.md) | [Русский](README.ru.md) | [Español](README.es.md)**

# Agent Reach RS (`agent-reach-rs`)

> **AIエージェント向け Pure-Rust メディア・Web・マルチチャンネルデータ取得エンジン**

`agent-reach-rs` は、AIエージェント（Hermes, Claude, Codex, OpenCode）が外部ウェブサイト、SNS、学術データベース、メディアファイルから信頼性の高いデータを高速かつ独立して取得できるようにする、モジュール式 Rust エコシステムです。

---

## 🎯 1. 目的と機能

- **外部 FFmpeg バイナリ非依存 (`MediaInspector`):** 外部 `ffmpeg.exe` に依存せず、`symphonia` (v0.5) ライブラリを用いて MP3, WAV, AAC, FLAC, OGG, MKV などの音声・メディア形式を Pure-Rust で直接解析。
- **14のマルチチャンネルリーダー:**
  - **SNS・Web:** Twitter/X (Nitter / GraphQL), Reddit API, Bilibili, 小紅書 (XHS), V2EX, 雪球, LinkedIn, 小宇宙.
  - **学術・コード:** Turath (イスラム法・写本データベース), GitHub REST API, RSS/Atom フィード.
  - **検索エンジン:** Exa AI セマンティック検索, DuckDuckGo HTML エクストラクター, Jina Web Reader.
- **5次元エピステミックベクトルエンジン (`agent-reach-graph`):** Turso SQLite (0.7.2) に基づく本体論・美学・認識論・倫理・言語次元マトリックス。
- **MCP サーバー統合:** Model Context Protocol (MCP) 標準に完全準拠した JSON-RPC CLI およびサーバーコンポーネント。

---

## 🏗️ 2. アーキテクチャとモジュール

```text
agent-reach-rs/
├── Cargo.toml                    # ワークスペース設定 (symphonia, tokio, reqwest)
├── crates/
│   ├── agent-reach-core/        # コア型定義、MediaInspector、エラー処理、設定
│   ├── agent-reach-channels/    # 14チャンネルリーダーの実装 (YouTube, Turath, RSS など)
│   ├── agent-reach-mcp/         # MCP JSON-RPC サーバードライバー
│   └── agent-reach-cli/         # CLI クライアント (バイナリ: agent-reach)
└── harness/                     # 自動テスト・査定検証ゲート
```

---

## 🚀 3. インストールとセットアップ

### 前提条件
- **Rust ツールチェーン:** Rust 1.75+ (`cargo` および `rustc` がインストールされていること)。
- **外部依存関係:** なし (FFmpeg バイナリ、Python、Node.js は不要)。

### ビルド
```bash
# リポジトリのクローン
git clone https://github.com/Ercaner1988/agent-reach-rs.git
cd agent-reach-rs

# ワークスペースのビルド
cargo build --release
```

ビルドされたバイナリは `target/release/agent-reach.exe` に生成されます。

---

## 📖 4. 使用方法と例

### A. Pure-Rust メディア解析 (`MediaInspector` API)
```rust
use agent_reach_core::MediaInspector;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ffmpeg.exe を呼び出さずに音声ファイルを直接解析
    let meta = MediaInspector::inspect_file("sample_audio.mp3")?;
    
    println!("コーデック: {}", meta.codec_name);
    println!("サンプルレート: {} Hz", meta.sample_rate);
    println!("チャンネル数: {}", meta.channels);
    println!("再生時間: {:.2} 秒", meta.duration_seconds);
    
    Ok(())
}
```

### B. CLI の使用
```bash
# Exa セマンティック検索の実行
agent-reach --channel exa search "Max Weber legal rationalization"

# Turath データベースからの文献取得
agent-reach --channel turath read --book 124 --page 45

# RSS フィードの取得
agent-reach --channel rss fetch "https://news.ycombinator.com/rss"
```

---

## 🛡️ 5. 品質ゲートとテスト

100% の合格率を義務付ける6つの厳格な検証ゲートによって保護されています。

```bash
# ワークスペース全体のテスト実行 (41/41 グリーンゲート)
cargo test --workspace
```

- **`agent-reach-core`:** 10/10 テスト合格 (Pure-Rust メディア解析を含む)。
- **`agent-reach-channels`:** 28/28 テスト合格。
- **`search_gauntlet`:** 3/3 査定ゲート検証済み。

---

## 👥 6. 貢献者

| 氏名 / 識別子 | 役割と貢献 | メトリクス |
| :--- | :--- | :--- |
| **Ercan Er** | リードアーキテクト兼プロジェクトオーナー (Rust アーキテクチャ) | 38 コミット, コアコードベース |
| **Mihenk** | コード監査人兼査定ゲートキーパー | 査定承認 & Gauntlet 監査 |
| **El-Kassâm** | エージェント開発者 (MediaInspector, Pure-Rust 統合) | 12 コミット, メディア & テストスイート |
| **GitHub Copilot** | 補助的コード補全 | ペアアシスタント |
| **Hermes** | エージェントオーケストレーションエンジン | エージェント実行環境 |

---

## 📄 7. ライセンス

本プロジェクトは **MIT ライセンス** の下で公開されています。詳細は `LICENSE` ファイルを参照してください。
