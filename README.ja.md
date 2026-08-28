**🌍 [Türkçe](README.md) | [English](README.en.md) | [العربية](README.ar.md) | [日本語](README.ja.md)**

---

# 👁️ Agent Reach (Rust)

**AIエージェントのための100% Pure-Rust製 Web閲覧＆セマンティック検索レイヤー**

> **注記:** 本プロジェクトは、[Agent Reach](https://github.com/Panniantong/agent-reach) Python版の完全なRust再実装です（アップストリームへの貢献ではなく、独立した新規実装）。目的: Python依存ゼロ、単一バイナリ、純粋なRustコンパイル、高速化。

---

## 🎯 ロードマップと完了項目

### 完了したコアコンポーネント
- [x] **ワークスペース骨格** — 4クレート (`core`, `channels`, `mcp`, `cli`)
- [x] **Web チャンネル** — Jina Reader (`r.jina.ai`) 統合
- [x] **RSS チャンネル** — RSS 2.0 および Atom フィードの取得と解析 (`fetch` + `parse`)
- [x] **Twitter** — `twitter-cli` (認証対応), Nitter (匿名)
- [x] **YouTube** — `yt-dlp` (メタデータ, 文字起こし, 検索)
- [x] **GitHub** — `gh` CLI (検索緩和ラダー), GitHub REST API
- [x] **Reddit** — Reddit API (OAuth2), PRAW (Python)
- [x] **中国SNSおよび金融** — Bilibili, 小紅書 (Xiaohongshu), V2EX, 雪球 (Xueqiu), 小宇宙 (Xiaoyuzhou)
- [x] **ビジネス＆検索** — LinkedIn, Exa Search, DuckDuckGo (HTML検索)
- [x] **CLI** — `install`, `configure`, `doctor`, `skill`, `transcribe`, `execute`
- [x] **MCP サーバー** — stdio JSON-RPC、5ツール (`web_read`, `rss_fetch`, `rss_parse`, `exa_search`, `agent_reach_execute`)
- [x] **マルチプラットフォームビルド** — Windows, Linux, macOS (`cargo-dist`)
- [x] **CI/CD パイプライン** — GitHub Actions CI/CD および自動テストゲート (`harness/` (Rust))

ロードマップ資料: [`docs/YOL-HARITASI-KAYNAKLAR.md`](docs/YOL-HARITASI-KAYNAKLAR.md)

### 既知の制限 (未実装)
- `agent-reach-graph` (セマンティックマインドマップ) クレートは計画段階で、リポジトリにはまだ存在しません。
- YouTube の `rustube` バックエンドはプレースホルダーです。動作する経路は `yt-dlp` サブプロセスです。
- Twitter の Nitter フォールバックはプレースホルダー レベルの単純な HTML 抽出です。
- `configure --from-browser` (ブラウザからの Cookie 抽出) は未実装です。
- `install` は設定ディレクトリの準備のみを行い、`gh` や `yt-dlp`、`twitter-cli` などの外部ツールはインストールしません。
- Reddit には OAuth2 認証情報 (`reddit_client_id`, `reddit_client_secret`) が必要です。

---

## 🏗️ アーキテクチャ

```
agent-reach-rs/
├── crates/
│   ├── agent-reach-core/      # 設定, バックエンド/チャンネル trait, カセットキャッシュ
│   ├── agent-reach-channels/  # 15プラットフォームリーダー (web, youtube, twitter, github, ...)
│   ├── agent-reach-mcp/       # MCP stdio JSON-RPC サーバー
│   └── agent-reach-cli/       # Clap CLI (install, doctor, skill, execute)
├── harness/                   # 自動監査ゲートおよびカセットキャッシュストア
└── Cargo.toml                 # ワークスペースルート
```

### バックエンド戦略

各チャンネルは複数のバックエンドを定義します（優先選択肢 ＋ フォールバック）：
- **Twitter:** `twitter-cli` $\rightarrow$ フォールバック: `Nitter`
- **Reddit:** `Reddit API` $\rightarrow$ フォールバック: `PRAW`
- **YouTube:** `yt-dlp` サブプロセス (メタデータ, 文字起こし, 検索); `rustube` バックエンドはプレースホルダー
- **GitHub:** `gh` CLI (クォートなし個別単語分割) $\rightarrow$ フォールバック: `GitHub REST API`

---

## 🚀 インストール

### ソースからのビルド

```bash
git clone https://github.com/Ercaner1988/agent-reach-rs.git
cd agent-reach-rs
cargo build --release
./target/release/agent-reach --help
```

### 安定版のインストール

```bash
cargo install agent-reach-cli
agent-reach install --env=auto
agent-reach doctor  # ヘルスおよび依存関係チェック
```

---

## 📖 使い方と実行

### Web チャンネル — 単一ページの読み込み

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

### バッチタスク実行 (`tasks.json`)

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

## 🛡️ 自動テストゲート

プロジェクトへの変更はすべて、6つの完全無料テストゲート (`harness/` (Rust)) を通過します：

```bash
cargo run --manifest-path harness/Cargo.toml -- gates
```

- **ゲート 1 (ビルド):** `cargo build --workspace`
- **ゲート 2 (Clippy):** `cargo clippy --workspace --all-targets -- -D warnings`
- **ゲート 3 (単体テスト):** `cargo test --workspace`
- **ゲート 4 (フォーマット):** `cargo fmt --check`
- **ゲート 5 (カンニング防止スキャン):** 正解データのフレーズがソースコードに漏洩するのを防ぐ自動スキャン
- **ゲート 6 (ゲートキーパー):** 審判ファイルの Git リファレンス検証

---

## 👥 貢献者 (Contributors)

詳細なリストについては [`CONTRIBUTORS.md`](CONTRIBUTORS.md) を参照してください。
- **Ercan ER** ([@Ercaner1988](https://github.com/Ercaner1988)) — プロジェクトリード＆アーキテクト
- **Kassam** (Hermes Agent / Nous Research) — AIピア＆共同開発者
- **Mihenk** (Claude Opus 5 / Anthropic) — 審判＆アーキテクチャ査読者
- **Devin AI** — 自動化コントリビューター
