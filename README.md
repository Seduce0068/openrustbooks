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
│       ├── 0004-zip-archive-reader.md
│       ├── 0005-epub-body-reading.md
│       ├── 0006-epub-table-of-contents.md
│       ├── 0007-epub-toc-fragments.md
│       ├── 0008-epub-cover-and-chapter-title.md
│       ├── 0009-epub-landmarks-and-page-list.md
│       ├── 0010-epub-wasm-bindings.md
│       └── 0011-web-epub-reader.md
├── core/                         # 共通コアライブラリ (native + wasm)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── error.rs
│   │   ├── document.rs           # ドキュメントモデル
│   │   ├── io.rs                 # ストレージ抽象化 (local はネイティブ専用)
│   │   ├── config.rs             # 設定管理
│   │   ├── archive.rs            # ZIPアーカイブ閲覧 (native + wasm)
│   │   ├── epub.rs               # EPUB本文・目次・表紙・ナビゲーションの読み出し (native + wasm)
│   │   ├── sanitize.rs           # EPUB XHTML の許可リストサニタイズ (native + wasm)
│   │   ├── ai_markdown/          # AI Markdownパイプライン
│   │   │   ├── mod.rs
│   │   │   ├── client.rs         # AIプロバイダ抽象化 (ネイティブ専用)
│   │   │   ├── lint.rs           # rumdlによるlint (native + wasm)
│   │   │   ├── prompt.rs         # プロンプトテンプレート
│   │   │   ├── diff.rs           # 差分計算
│   │   │   └── pipeline.rs       # 高レベルサービス (ネイティブ専用)
│   │   └── wasm.rs               # Wasmバインディング（EPUB読み出し API）
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
| **デスクトップ** | Tauri 2.12 (OS WebView / wry) |
| **Web** | Leptos 0.8 (CSR), Wasm |
| **Markdown処理** | `pulldown-cmark`, `comrak`, `rumdl` (Wasm対応) |
| **AI/LLM** | OpenAI API互換, Ollama (ローカル) |
| **設定** | `figment` (TOML + 環境変数) |
| **ストレージ** | ローカルFS, Google Drive (予定) |

## 主な機能

- **EPUB 閲覧 (Web, 実装済み)**: 章 HTML の許可リストサニタイズ表示、
  画像・表紙の Blob URL 表示、目次・章移動、章内アンカー移動、
  本文内の相対リンクによる章移動
- **ドキュメント閲覧 (設計段階)**: EPUB, ZIP, PDF, TXT, Markdown
- **テキスト編集 (設計段階)**: TXT, Markdown (CodeMirror 6ベース)
- **AI支援 (設計段階)**: Markdown生成・修正・リファクタリング +
  `rumdl`品質保証
- **クロスプラットフォーム**: デスクトップ (Tauri) + ブラウザ (Wasm)
- **フォルダ操作 (設計段階)**: 既存フォルダ直接参照, インポート不要
- **クラウド連携 (設計段階)**: Google Drive (手動保存から開始)

## 開発セットアップ

### 前提条件

- Rust 1.94+ (2024 edition)。`rumdl` の MSRV に合わせる
- Trunk (`cargo install trunk`, または各ディストリビューションのパッケージ)
- `wasm32-unknown-unknown` ターゲット (`rustup target add wasm32-unknown-unknown`)
- `wasm-bindgen` CLI (Trunk は通常自動取得する。取得できない環境では
  `cargo install wasm-bindgen-cli --version <Cargo.lock の wasm-bindgen と同一>`)
- Tauri CLI: `cargo install tauri-cli` (デスクトップアプリ開発時)

### ビルド

```bash
# ワークスペース全体 (ネイティブ)
cargo check --all-targets --locked
cargo test --all-targets --locked

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

### CI

push（`master`）と pull request では、GitHub Actions がローカルの 5 ゲートと
同じ検証を実行する。検証本体は個人リポジトリ共通の再利用ワークフロー
（`Seduce0068/workflows` の `rust-gates.yml`、タグ `v1`）に委譲しており、
`.github/workflows/ci.yml` は Tauri のビルド依存と wasm32 ゲートを渡す
呼び出し側になっている。desktop を含むワークスペース全体を Linux 向けに
検証する。

```bash
cargo fmt --all -- --check
cargo check --all-targets --locked
cargo test --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
cargo check -p openrustbooks-core --target wasm32-unknown-unknown --locked
cargo check -p openrustbooks-web --target wasm32-unknown-unknown --locked
```

### 手動の E2E 検証

Web 版リーダーの表示と移動は、ヘッドレス Chrome を使った
`tools/verify-web-reader.py` で確認できる。Python 3.10 以降（`websockets`）と
Chrome/Chromium が必要で、サンプル EPUB は一時ディレクトリに生成する。

```bash
cd web && trunk build --dist dist
cd .. && python3 tools/verify-web-reader.py --dist web/dist
```

## ドキュメント

- [要件定義](docs/requirements.md) - ユーザー要件, 未決定事項, 検証条件
- [アーキテクチャ設計](docs/architecture.md) - 構成案, 開発順序, 技術選定
- [ADR-0001: OS WebView採用](docs/decisions/0001-use-system-webview.md)
- [ADR-0002: AI Markdownパイプライン](docs/decisions/0002-ai-markdown-pipeline.md)
- [ADR-0003: Native/Wasm ターゲット分離](docs/decisions/0003-native-wasm-target-split.md)
- [ADR-0004: ZIPアーカイブ閲覧](docs/decisions/0004-zip-archive-reader.md)
- [ADR-0005: EPUB本文の読み出し](docs/decisions/0005-epub-body-reading.md)
- [ADR-0006: EPUB目次の読み出し](docs/decisions/0006-epub-table-of-contents.md)
- [ADR-0007: EPUB目次フラグメント](docs/decisions/0007-epub-toc-fragments.md)
- [ADR-0008: EPUB表紙と章タイトル](docs/decisions/0008-epub-cover-and-chapter-title.md)
- [ADR-0009: EPUB landmarksとpage-list](docs/decisions/0009-epub-landmarks-and-page-list.md)
- [ADR-0010: EPUB読み出しAPIのWasm公開](docs/decisions/0010-epub-wasm-bindings.md)
- [ADR-0011: Web版EPUBテキストリーダー](docs/decisions/0011-web-epub-reader.md)
- [ADR-0012: EPUB HTMLのサニタイズ表示](docs/decisions/0012-epub-html-display.md)

## ライセンス

MIT License - see [LICENSE](LICENSE) for details.
