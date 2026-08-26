**🌍 [Türkçe](README.md) | [English](README.en.md) | [العربية](README.ar.md) | [日本語](README.ja.md)**

---

# 👁️ Agent Reach (Rust)

**AIエージェントのための100% Pure-Rust製 Web閲覧＆セマンティック検索レイヤー**

> **注記:** 本プロジェクトは、[Agent Reach](https://github.com/Panniantong/agent-reach) Python版の完全なRust再実装です（アップストリームへの貢献ではなく、独立した新規実装）。目的: Python依存ゼロ、単一バイナリ、純粋なRustコンパイル、高速化。

---

## 🎯 ロードマップと完了項目

### 完了したコアコンポーネント
- [x] **ワークスペース骨格** — 5クレート (`core`, `channels`, `graph`, `mcp`, `cli`)
- [x] **Web チャンネル** — Jina Reader (`r.jina.ai`) 統合
- [x] **RSS チャンネル** — RSS 2.0 および Atom フィードの取得と解析 (`fetch` + `parse`)
- [x] **Twitter** — `twitter-cli` (認証対応), Nitter (匿名)
- [x] **YouTube** — `yt-dlp` (フル抽出), `rustube` (メタデータ)
- [x] **GitHub** — `gh` CLI (3段階検索緩和ラダー), GitHub REST API
- [x] **Reddit** — PRAW (Python), Reddit API (OAuth2)
- [x] **中国SNSおよび金融** — Bilibili, 小紅書 (Xiaohongshu), V2EX, 雪球 (Xueqiu), 小宇宙 (Xiaoyuzhou)
- [x] **ビジネス＆検索** — LinkedIn, Exa Search, DuckDuckGo (HTML検索)
- [x] **セマンティックマインドマップ** — `agent-reach-graph` (Pure-Rust 5次元認識論ベクトルエンジン)
- [x] **CLI** — `install`, `configure`, `doctor`, `skill`, `transcribe`, `execute`
- [x] **MCP サーバー** — stdio JSON-RPC インターフェース、14チャンネルへのアクセス
- [x] **マルチプラットフォームビルド** — Windows, Linux, macOS (`cargo-dist`)
- [x] **CI/CD パイプライン** — GitHub Actions CI/CD および自動テストゲート (`harness/` (Rust))

詳細マップ: [`docs/HARITA.md`](docs/HARITA.md) (Yolbulucu/Wayfinder 構造)

---

## 🏗️ アーキテクチャ

```
agent-reach-rs/
├── crates/
│   ├── agent-reach-core/      # 設定, バックエンド/チャンネル trait, カセットキャッシュ
│   ├── agent-reach-channels/  # 14プラットフォームリーダー (web, youtube, twitter, github, ...)
│   ├── agent-reach-graph/     # Pure-Rust セマンティックグラフ＆認識論ベクトルエンジン
│   ├── agent-reach-mcp/       # MCP stdio JSON-RPC サーバー
│   └── agent-reach-cli/       # Clap CLI (install, doctor, skill, execute)
├── harness/                   # 自動監査ゲートおよびカセットキャッシュストア
└── Cargo.toml                 # ワークスペースルート
```

### バックエンド戦略

各チャンネルは複数のバックエンドを定義します（優先選択肢 ＋ フォールバック）：
- **Twitter:** `twitter-cli` $\rightarrow$ フォールバック: `Nitter`
- **Reddit:** `Reddit API` $\rightarrow$ フォールバック: `PRAW`
- **YouTube:** `rustube` (メタデータ) + `yt-dlp` サブプロセス (フル抽出)
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
