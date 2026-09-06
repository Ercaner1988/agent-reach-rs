**🌍 [Türkçe](README.md) | [English](README.en.md) | [العربية](README.ar.md) | [日本語](README.ja.md) | [中文](README.zh.md) | [Русский](README.ru.md) | [Español](README.es.md)**

# Agent Reach RS (`agent-reach-rs`)

> **AIエージェントにインターネット全体を見渡す目を与える、純粋なRustの読み取りエンジン**

`agent-reach-rs`は、AIエージェント（Hermes、Claude、Codex、OpenCode）がウェブページ、ソーシャルネットワーク、法律および学術データベース、ローカルの音声/動画ファイルから確実にデータを読み取るためのモジュール式のRustワークスペースです。

このドキュメントが基づく測定結果：`git rev-parse --short HEAD` → `dbe3b0c`、`cargo test --workspace` → **58 passed, 0 failed, 1 ignored**、`harness/gates` → **8 gates green**（2026年9月7日）。

---

## 🎯 1. 目標と完成した機能

- **17 channelの読み取り機能** — 各チャネルは、1つ以上のバックエンドを試行します。1つが失敗した場合、次を試行します（CLI → API → スクレイパーと段階的に移行）。
- **外部のFFmpeg依存なし（`MediaInspector`）:** MP3、WAV、AAC、FLAC、OGG、MKVファイルは、外部の`ffmpeg`バイナリなしで`symphonia` 0.5を使用して解析されます。`format_name`フィールドは常に`symphonia-native`を返します。
- **チャネルの切り替え（`disabled_channels`）:** 不要なソースは、設定または環境変数を使ってオフにできます。無効化されたチャネルは、**壊れたチャネルとは異なるエラー**を生成します。一方は決定によるものであり、もう一方は追跡すべき障害です。
- **ペイロードの検証（`require_payload`、ADR 0004）:** ペイロードコンテンツを持たないままHTTP 200を返すレスポンス（ログイン時の壁、オフライン通知、404ランディングページ）をキャッチし、静かに成功として扱わないようにします。
- **プッシュ前の品質ゲート（`.githooks/pre-push`）:** プッシュ前にローカルで8つの品質ゲートを実行し、フォーマットやビルドのチェックに失敗した場合はプッシュを停止します。
- **CIパリティゲート（`gate_ci_parity`）:** CIワークフローで定義されたすべての`cargo`チェックが、ローカルのharnessテストゲートに存在することを検証します。
- **カセットシステム:** レスポンスは記録および再生されます。失敗も記録されるため、ネットワークなしで「測定外」の経路を試験できます。デフォルトではオフです。
- **MCPサーバー:** Model Context Protocol 2024-11-05、stdio JSON-RPC、5つのツールをサポート。
- **ローカルの文字起こし（`transcribe`）:** ローカルの音声/動画ファイルは、GroqまたはOpenAIを介してWhisper（`whisper-large-v3-turbo`）で文字起こしされます。
- **審判によって保護された測定:** ゴールデンセット、合格基準、そして改ざんチェックは別々のファイルに存在します。測定する側は測定される側を変更できません。

### Channels (17)

| Channel | Actions | Note |
| :--- | :--- | :--- |
| `web` | `read` | Jina Readerによるページテキスト |
| `rss` | `fetch`, `parse` | RSS 2.0 + Atom; Substackの出版物もこのチャネルを使用します |
| `twitter` | `search`, `timeline`, `thread` | 2つのバックエンド |
| `youtube` | `metadata`, `transcript`, `search` | 2つのバックエンド |
| `github` | `repo`, `issue`, `pr`, `search` | `gh` CLI + REST API |
| `reddit` | `subreddit`, `search`, `post` | PRAW + REST API |
| `bilibili` | `video`, `info`, `search` | 2つのバックエンド |
| `xiaohongshu` | `note`, `search` | Webバックエンド |
| `linkedin` | `profile`, `company`, `search` | 認証情報が必要です |
| `v2ex` | `topic`, `node`, `hot`, `latest` | Open API |
| `xueqiu` | `quote`, `stock`, `timeline`, `search` | 市場データ |
| `xiaoyuzhou` | `podcast`, `episode` | ポッドキャスト |
| `exa` | `search` | セマンティック検索、キーが必要です |
| `duckduckgo` | `search` | HTMLエクストラクタ、キーなし |
| `turath` | `search`, `book`, `author`, `page` | イスラムの古典的著作。**巻とページ番号**を返します |
| `mevzuat` | `mevzuat`, `metin`, `fihrist` | トルコの法律（`mevzuat.gov.tr`）、法律番号で直接照会します |
| `uinjkt` | `identify`, `journals`, `articles`, `record` | UIN Jakarta イスラム学術ジャーナル（OAI-PMH 2.0、オープンアクセス） |

### 次にキューに入れられているソース

`docs/YOL-HARITASI-KAYNAKLAR.md`で、測定されたエンドポイントについて文書化されています：
**UYAP Emsal** と **Yargıtay Karar Arama**（どちらもキーレスJSON、`aramalist` + `getDokuman`）、**Danıştay**（エンドポイントは有効、検索ボディは未解決）、**Pew Research**（`robots.txt`がエージェントのアクセスを明示的に許可、`ai-input=yes`）、**Lexpera**（サブスクリプションの壁 + 必須の 10-second crawl delay → 人間ゲートモデル）、およびWikipedia、Stanford Encyclopedia of Philosophy、TDV Encyclopedia of Islam、OpenAlex、Crossref、トルコ国立論文センター。

---

## 🏗️ 2. アーキテクチャとモジュール

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

**なぜ`harness`はワークスペースの外にあるのか：** ゲートを実行するツールが測定対象のツリーと一緒にコンパイルされた場合、コードの破損が審判ツールも道連れにしてしまうためです。

### コアの型

| Type | Responsibility |
| :--- | :--- |
| `Channel` | ソースの名前、アクション、およびエグゼキューター |
| `Backend` | チャネル下の単一パス。`BackendStatus`を通じて正常性を報告します |
| `Config` | `~/.agent-reach/config.yaml`の読み書きし、`channel_enabled`を適用します |
| `Cassette` | レスポンスの記録と再生（`AGENT_REACH_CASSETTE`） |
| `MediaInspector` | 純粋なRustのメディアパーサー（`symphonia`） |
| `HealthCheck` | `doctor`出力の背後にあるデータ型 |

---

## 🚀 3. インストールと設定

### 前提条件

- **Rust 1.75+** （`cargo` と `rustc`）。
- **外部バイナリは不要:** FFmpeg、Python、Node.jsは必須ではありません。特定のチャネル向けのオプションのバックエンド（`gh` CLI、`praw`、`BBDown`）は、インストールされている場合に使用されます。そうでない場合、チャネルは他のバックエンドへとフォールスルーします。

### ビルド

```bash
git clone https://github.com/Ercaner1988/agent-reach-rs.git
cd agent-reach-rs
cargo build --release
```

生成されるバイナリ：`target/release/agent-reach`（Windowsでは`agent-reach.exe`）。

> **Note:** このリポジトリは、ダウンロード可能な事前ビルドされたバイナリを公開していません。`dist-workspace.toml`では`installers = []`と設定されており、リリースのアセットはアーカイブとしてのみ生成されます。ソースからのビルドが唯一のパスです。

### 初回設定

```bash
agent-reach install                      # Creates the ~/.agent-reach directory
agent-reach configure github_token <tk>  # Set a key
agent-reach configure --json             # View current settings
agent-reach doctor                       # Channel health check
agent-reach doctor --json                # Machine-readable health report
```

`install`コマンドは外部ツールをインストール**しません**。単に設定ディレクトリを準備するだけです。

### チャネルの無効化

`~/.agent-reach/config.yaml`:

```yaml
disabled_channels:
  - reddit
  - linkedin
```

または、ファイルを変更せずに単一の実行を行う場合：

```bash
AGENT_REACH_DISABLED_CHANNELS=reddit,linkedin agent-reach execute --task-file tasks.json
```

名前の比較では、大文字と小文字、および空白は無視されます。プレフィックスマッチングはありません（`red`と記述しても`reddit`は無効になりません）。これはCLIとMCPパスの両方に適用されます。

### 認識される設定キー

`twitter_auth_token` · `twitter_ct0` · `groq_api_key` · `openai_api_key` · `github_token` · `reddit_client_id` · `reddit_client_secret` · `reddit_user_agent` · `linkedin_username` · `linkedin_password` · `exa_api_key` · `proxy` · `disabled_channels`

---

## 📖 4. 使用法と例

### A. コマンドライン

`agent-reach --help`からのサブコマンド：

```text
install     Set up the config directory (external tools are not installed)
configure   Set a config value or auto-extract from browser
doctor      Check platform availability
skill       Manage agent skill registration
execute     Execute tasks from a JSON file
transcribe  Transcribe a local audio/video file
```

### B. タスクの実行

チャネルはタスクファイルを介して駆動されます（CLIを介して全17のチャネルがサポートされています）：

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

出力（`log.json`）のスキーマ：

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

### C. ローカルの文字起こし

```bash
agent-reach transcribe recording.mp3
agent-reach transcribe lecture.mkv --provider groq --output lecture.txt
```

ローカルファイルのみが受け付けられます。`groq_api_key`も`openai_api_key`も設定されていない場合、コマンドはそれを明確に伝え、`configure`を案内します。

### D. 純粋なRustのメディア解析（`MediaInspector`）

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

### E. MCPサーバー

`agent-reach-mcp`バイナリは、stdioを介してJSON-RPC（プロトコル`2024-11-05`）を話し、5つのツールを公開します：

| Tool | Parameters |
| :--- | :--- |
| `web_read` | `url` |
| `rss_fetch` | `url` |
| `rss_parse` | `xml` |
| `exa_search` | `query`, `num_results` (1–10) |
| `agent_reach_execute` | `channel`, `action`, `args[]` |

`agent_reach_execute`は次の12のチャネルを受け入れます：`bilibili`、`duckduckgo`、`github`、`linkedin`、`reddit`、`turath`、`twitter`、`v2ex`、`xiaohongshu`、`xiaoyuzhou`、`xueqiu`、`youtube`。`web`、`rss`、および`exa`は、それら自身のツールを介してアクセスされるため、MCP上で14のチャネルが利用可能です。

> **Honesty note:** 17のチャネルすべてがCLIの`execute`を通じて実行されますが、`mevzuat`と`uinjkt`は、まだMCPの`agent_reach_execute`列挙型（enum）または`doctor`コマンドに追加されておらず、次のリリースで予定されています。

Hermes / Claude Desktopの設定：

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

## 🛡️ 5. 品質ゲートとテスト

### 測定されたテスト状態

```bash
cargo test --workspace
```

このリポジトリに対して2026年9月7日に実行された実際の出力：

| Suite | Result |
| :--- | :--- |
| `agent_reach_channels` (lib) | 39 passed · 0 failed |
| `agent_reach_core` (lib) | 16 passed · 0 failed |
| `search_gauntlet` (integration) | 3 passed · 0 failed · 1 ignored |
| **Total** | **58 passed · 0 failed · 1 ignored** |

単一のスキップされた（ignored）テストは有効なネットワークを必要とするため、`--ignored`を使用して手動で実行してください。

### 審判システムと8つのゴールデンゲート

`harness`は、`cargo run --manifest-path harness/Cargo.toml -- gates`を介して8つのゲートを実行する個別のクレートです：

1. **build** — ワークスペースのコンパイル検証。
2. **clippy** — 警告ゼロの強制（`-D warnings`）。
3. **unit tests** — すべての単体テストの合格。
4. **formatting** — `cargo fmt`によるフォーマットへの準拠。
5. **answer-key grep** — ゴールデンセットの解答文字列がソースコードにコピーされていないことを確認します（138のフレーズがスキャンされます）。
6. **referee watch** — 審判ファイルが変更されていないことを検証します。
7. **criteria wiring** — 合格基準が能動的にチェックされていることを確認します。
8. **ci parity** — ローカルのharnessテストゲート内に、すべてのCIチェックが存在することを保証します。

しきい値は`harness/kabul.json`にあり、ゴールデンセットとは**別々**になっています（`min_recall_ratio: 0.9`、`max_zero_results: 0`）。ゴールデンセットは、`crates/agent-reach-channels/tests/golden_search.json`の中に**24のケース**を保持しています。

**公然と記録されたギャップ:** 24のケースすべてが`github`を対象としています。チケットAの条件である「新しいチャネルは、ゴールデンセットの少なくとも1つのケースで勝利するまでは機能していると見なされない」は、`turath`、`mevzuat`、および`uinjkt`においては**まだ満たされていません**。これらのチャネルはライブで測定されましたが、ゴールデンセットには入っていません。

### 測定の倫理

- `docs/adr/0003-a-refusal-is-not-a-verdict.md`により：トランスポート層の拒否（HTTP 429、202）は、**機能の欠落としてカウントされません**。それは`Not measured`（測定外）としてマークされます。
- `docs/adr/0004-a-200-is-not-a-payload-verdict.md`により：実際のデータのないHTTP 200レスポンス（`require_payload`）は、エラーとして扱われます。

---

## 👥 6. コントリビューター

以下のカウントは、`git shortlog -sne --all`からのものです（2026年9月7日、合計77のコミット）。

| Name / Identity | Role and Contributions | Measurement |
| :--- | :--- | :--- |
| **Ercan ER** ([@Ercaner1988](https://github.com/Ercaner1988)) | プロジェクトの所有者および主任アーキテクト。Rustアーキテクチャ、17チャネルの設計、ソース生成、測定倫理（ADR 0001–0004） | **74 commits** (43 + 22 + 9、3つの署名が統合) |
| **copilot-swe-agent[bot]** ([@copilot-swe-agent[bot]](https://github.com/apps/copilot-swe-agent)) | PR #2: CI `cargo fmt` およびLintとフォーマットの修正 | **2 commits** |
| **Devin AI** ([@devin-ai-integration[bot]](https://github.com/apps/devin-ai-integration)) | PR #1: クロスプラットフォームのPythonパス検出、LinkedIn/Redditのインジェクション保護、CLIルーティング | **1 commit** |
| **Kassam** (Hermes agent) | AIのピア開発者。`MediaInspector`、`turath`チャネル、チャネルスイッチ、7言語ドキュメントスイート | Ercan ERの署名配下でコミット |
| **Mihenk** (Claude Opus 5) | アーキテクチャのレビューアーおよび審判。ADR 0001–0004、審判ロック、Tickets A · B · C、ペイロード検証 | Ercan ERの署名配下でコミット |
| **GitHub Copilot** (Editor Assistant) | エディタ内コード補完。反復的なRustパターン、`match`アーム、テストの足場作り | 独立したgitの署名なし |
| **ZAI GLM 5.3** (Zhipu AI) | 検索の階層化とチャネル最適化の提案（commit `139b713`） | Ercan ERの署名配下でコミット |

**Honesty note:** Kassam、Mihenk、ZAI GLM 5.3、およびGitHub Copilotエディタアシスタントは、個別のgit署名でコミットを行いません。これらの変更はErcan ERの署名の下に記録されています。PRを直接開くGitHub Bots（`copilot-swe-agent[bot]`: 2、`Devin AI`: 1）は、独自の署名で表示されています。

---

## 📄 7. ライセンス

このプロジェクトは**MIT License**の下でライセンスされています — 詳細は`LICENSE`ファイルを参照してください。
Copyright: 2026 Ercan ER.

Documentation: `CONTEXT.md` (glossary), `docs/adr/` (0001–0004 architecture decisions), `docs/YOL-HARITASI-KAYNAKLAR.md` (source generations), `docs/channels/mevzuat.md`, `docs/channels/KANAL-EKLEME.md`, `docs/integration/mcp.md`, `docs/channels/rss.md`.
