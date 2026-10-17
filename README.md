# openrustbooks

Rustを活用する、ブラウザでも使えるリーダー兼テキストエディタ。

## プロジェクト構成

```
openrustbooks/
├── Cargo.toml                    # ワークスペース設定
├── Cargo.lock                    # アプリケーションのため追跡する
├── README.md                     # プロジェクト概要
├── LICENSE                       # MIT License
├── docs/
│   ├── requirements.md           # 要件定義
│   ├── architecture.md           # アーキテクチャ設計
│   └── decisions/                # ADR (Architecture Decision Records)
│       ├── 0001-use-system-webview.md
│       ├── 0002-ai-markdown-pipeline.md
│       ├── 0003-native-wasm-target-split.md
│       └── 0004-zip-archive-reader.md
├── core/                         # 共通コアライブラリ (native + wasm)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── error.rs
│   │   ├── document.rs           # ドキュメントモデル
│   │   ├── io.rs                 # ストレージ抽象化 (local はネイティブ専用)
│   │   ├── config.rs             # 設定管理
│   │   ├── archive.rs            # ZIPアーカイブ閲覧 (native + wasm)
│   │   ├── ai_markdown/          # AI Markdownパイプライン
│   │   │   ├── mod.rs
│   │   │   ├── client.rs         # AIプロバイダ抽象化 (ネイティブ専用)
│   │   │   ├── lint.rs           # rumdlによるlint (native + wasm)
│   │   │   ├── prompt.rs         # プロンプトテンプレート
│   │   │   ├── diff.rs           # 差分計算
│   │   │   └── pipeline.rs       # 高レベルサービス (ネイティブ専用)
│   │   └── wasm.rs               # Wasmバインディング
│   ├── prompts/                  # プロンプトテンプレート
│   └── Cargo.toml
├── desktop/                      # Tauriデスクトップアプリ
│   ├── src/
│   │   ├── main.rs
│   │   ├── build.rs              # tauri-build
│   │   ├── state/                # アプリケーション状態
│   │   ├── window/               # ウィンドウ管理
│   │   └── commands/             # Tauriコマンド
│   ├── tauri.conf.json           # tauri-build がクレート直下を参照する
│   ├── icons/
│   └── Cargo.toml
└── web/                          # Wasmウェブアプリ (Leptos)
    ├── src/
    │   └── lib.rs
    ├── index.html                # Trunk エントリポイント
    ├── Trunk.toml
    ├── favicon.svg
    └── Cargo.toml
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

- Rust 1.85+ (2024 edition)
- Trunk (`cargo install trunk`, または各ディストリビューションのパッケージ)
- `wasm32-unknown-unknown` ターゲット (`rustup target add wasm32-unknown-unknown`)
- `wasm-bindgen` CLI (Trunk は通常自動取得する。取得できない環境では
  `cargo install wasm-bindgen-cli --version <Cargo.lock の wasm-bindgen と同一>`)
- Tauri CLI: `cargo install tauri-cli` (デスクトップアプリ開発時)

### ビルド

```bash
# ワークスペース全体 (ネイティブ)
cargo check --workspace
cargo test --workspace

# ブラウザ版 (Trunk が Wasm をビルドし web/dist に出力する)
cd web
trunk build          # 本番ビルドは trunk build --release
trunk serve --port 1420

# デスクトップアプリ開発 (beforeDevCommand が trunk serve を起動する)
cd desktop
cargo tauri dev
```

### 対応ターゲット

共通コアはネイティブと `wasm32-unknown-unknown` の両方でビルドできる。ネイティブ専用の依存と機能は ADR-0003 に従ってターゲット別に分離されている。

## ドキュメント

- [要件定義](docs/requirements.md) - ユーザー要件, 未決定事項, 検証条件
- [アーキテクチャ設計](docs/architecture.md) - 構成案, 開発順序, 技術選定
- [ADR-0001: OS WebView採用](docs/decisions/0001-use-system-webview.md)
- [ADR-0002: AI Markdownパイプライン](docs/decisions/0002-ai-markdown-pipeline.md)
- [ADR-0003: Native/Wasm ターゲット分離](docs/decisions/0003-native-wasm-target-split.md)

## ライセンス

MIT License - see [LICENSE](LICENSE) for details.