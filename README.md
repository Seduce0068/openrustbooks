# openrustbooks

Rustを活用する、ブラウザでも使えるリーダー兼テキストエディタ。

## プロジェクト構成

```
openrustbooks/
├── Cargo.toml                    # ワークスペース設定
├── README.md                     # プロジェクト概要
├── LICENSE                       # MIT License
├── docs/
│   ├── requirements.md           # 要件定義
│   ├── architecture.md           # アーキテクチャ設計
│   └── decisions/                # ADR (Architecture Decision Records)
│       ├── 0001-use-system-webview.md
│       └── 0002-ai-markdown-pipeline.md
├── crates/
│   ├── core/                     # 共通コアライブラリ (native + wasm)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── error.rs
│   │   │   ├── document.rs       # ドキュメントモデル
│   │   │   ├── io.rs             # ストレージ抽象化
│   │   │   ├── config.rs         # 設定管理
│   │   │   ├── ai_markdown/      # AI Markdownパイプライン
│   │   │   │   ├── mod.rs
│   │   │   │   ├── client.rs     # AIプロバイダ抽象化
│   │   │   │   ├── lint.rs       # rumdlによるlint
│   │   │   │   ├── prompt.rs     # プロンプトテンプレート
│   │   │   │   ├── diff.rs       # 差分計算
│   │   │   │   └── pipeline.rs   # 高レベルサービス
│   │   │   └── wasm.rs           # Wasmバインディング
│   │   ├── prompts/              # プロンプトテンプレート
│   │   └── Cargo.toml
│   ├── desktop/                  # Tauriデスクトップアプリ
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── state/            # アプリケーション状態
│   │   │   ├── window/           # ウィンドウ管理
│   │   │   └── commands/         # Tauriコマンド
│   │   ├── src-tauri/
│   │   │   └── tauri.conf.json
│   │   └── Cargo.toml
│   └── web/                      # Wasmウェブアプリ (Leptos)
│       ├── src/
│       │   └── lib.rs
│       ├── static/
│       │   └── index.html
│       └── Cargo.toml
```

## 技術スタック

| レイヤー | 技術 |
|----------|------|
| **共通コア** | Rust 2024, `rumdl` (markdownlint互換), `async-openai`, `tiktoken-rs` |
| **デスクトップ** | Tauri 2.0 (OS WebView / wry) |
| **Web** | Leptos 0.7 (CSR), Wasm |
| **Markdown処理** | `pulldown-cmark`, `comrak`, `rumdl` (Wasm対応) |
| **AI/LLM** | OpenAI API互換, Ollama (ローカル) |
| **設定** | `figment` (TOML + 環境変数) |
| **ストレージ** | ローカルFS, Google Drive (予定) |

## 主な機能 (設計段階)

- **ドキュメント閲覧**: EPUB, ZIP, PDF, TXT, Markdown
- **テキスト編集**: TXT, Markdown (CodeMirror 6ベース)
- **AI支援**: Markdown生成・修正・リファクタリング + `rumdl`品質保証
- **クロスプラットフォーム**: デスクトップ (Tauri) + ブラウザ (Wasm)
- **フォルダ操作**: 既存フォルダ直接参照, インポート不要
- **クラウド連携**: Google Drive (手動保存から開始)

## 開発セットアップ

### 前提条件

- Rust 1.80+ (2024 edition)
- Node.js 20+ (フロントエンド開発用)
- Tauri CLI: `cargo install tauri-cli`

### ビルド

```bash
# ワークスペース全体
cargo build --workspace

# デスクトップアプリ開発
cd crates/desktop
cargo tauri dev

# Webアプリ開発 (wasm-pack)
cd crates/web
wasm-pack build --target web --out-dir ../desktop/static/pkg
```

## ドキュメント

- [要件定義](docs/requirements.md) - ユーザー要件, 未決定事項, 検証条件
- [アーキテクチャ設計](docs/architecture.md) - 構成案, 開発順序, 技術選定
- [ADR-0001: OS WebView採用](docs/decisions/0001-use-system-webview.md)
- [ADR-0002: AI Markdownパイプライン](docs/decisions/0002-ai-markdown-pipeline.md)

## ライセンス

MIT License - see [LICENSE](LICENSE) for details.