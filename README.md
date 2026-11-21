# openrustbooks

Rustを活用する、ブラウザでも使えるリーダー兼テキストエディタ。

## プロジェクト構成

```text
openrustbooks/
├── Cargo.toml                    # ワークスペース設定
├── Cargo.lock                    # アプリケーションのため追跡する
├── README.md                     # プロジェクト概要
├── LICENSE                       # AGPL-3.0-or-later
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
| **Markdown処理** | `comrak` (Markdown→HTML), `ammonia` (sanitize), `rumdl` (lint, Wasm対応) |
| **AI/LLM** | OpenAI API互換, Ollama (ローカル) |
| **設定** | `figment` (TOML + 環境変数) |
| **ストレージ** | ローカルFS, Google Drive (予定) |

## 主な機能

- **EPUB 閲覧 (Web, 実装済み)**: 章 HTML の許可リストサニタイズ表示、SVG 画像のサニタイズ、
  画像・表紙の Blob URL 表示、目次・章移動、章内アンカー移動、
  本文内の相対リンクによる章移動、同一章内アンカー移動の履歴記録、
  読書位置（章・フラグメント・スクロール位置）の保存と復元、
  移動履歴（戻る・進む）、表示設定（文字サイズ・行間・ダーク表示、一括リセット）、
  保存データの削除と保持期間・件数上限による自動掃除、
  画像読み込み後の位置ずれ補正、画像の非同期デコード、キーボード操作と ARIA 属性（本文領域の章タイトルラベル含む）、強制カラーモード対応、
  読書位置のエクスポート・インポート（表示設定・履歴を含む・マージ・形式バージョン）、
  アプリ全体のダークテーマ、
  landmarks の移動 UI、page-list によるページ移動、履歴一覧からのジャンプと日時表示、履歴のクリアと個別削除（確認ダイアログ付き）、
  OS のカラースキーム追従、本文言語の反映（スクリーンリーダー対応）、
  移動履歴の永続化、タッチ操作（スワイプ、横・縦の閾値設定可能）、
  章タイトルの検索フィルター、巨大画像の警告とオプトインの縮小（JPEG 品質設定付き）
- **ドキュメント閲覧 (実装済み)**: EPUB, ZIP, Markdown
- **ドキュメント閲覧 (設計段階)**: PDF, TXT
- **テキスト編集 (ブラウザは Markdown のみ)**: textarea + Preview（CodeMirror 6 は未採用）、format 選択と Lint は desktop 向け。
- **desktop 設定保存 (Tauri)**: browser は読み取り専用 fail-closed、desktop は UI 起点 CAS と
  構造化エラーで接続済み（Tauri mock E2E 256/0）。実 WebView でも settings / get_config /
  update_config_patch / config.toml 永続化を検証済み（Tauri CLI 2.12.5、ADR-0282）。
  WebdriverIO による独立 E2E ゲートも運用中（ADR-0283、
  [WebdriverIO 実 WebView E2E](docs/runbook-webdriver.md)）。
- **AI支援 (設計段階)**: Markdown生成・修正・リファクタリング +
  `rumdl`品質保証
- **クロスプラットフォーム**: デスクトップ (Tauri) + ブラウザ (Wasm)
- **フォルダ操作 (設計段階)**: 既存フォルダ直接参照, インポート不要
- **クラウド連携 (設計段階)**: Google Drive (手動保存から開始)
- **初版スコープ**: 実装・検証済みの提供範囲と未決定の境界は
  [ADR-0232: 初版リリーススコープ](docs/decisions/0232-initial-release-scope.md) を参照。

## キーボードショートカット

リーダーでは次のショートカットを使える（アプリ内でも `?` で確認できる）。

| キー | 動作 |
| --- | --- |
| `[` / `]` | 前の章 / 次の章 |
| `+` / `-` | 文字サイズを大きく / 小さく |
| `d` | ダーク表示の切り替え |
| `Alt+←` / `Alt+→` | 履歴を戻る / 進む |
| `/` | 章フィルターにフォーカス |
| `?` | ショートカットヘルプを表示 |

## 開発セットアップ

### 前提条件

- Rust 1.95+ (2024 edition)。`parking_lot` / `lock_api` の MSRV に合わせる
- Trunk (`cargo install trunk`, または各ディストリビューションのパッケージ)
- `wasm32-unknown-unknown` ターゲット (`rustup target add wasm32-unknown-unknown`)
- `wasm-bindgen` CLI (Trunk は通常自動取得する。取得できない環境では
  `cargo install wasm-bindgen-cli --version <Cargo.lock の wasm-bindgen と同一>`)
- Tauri CLI: `cargo install tauri-cli` (デスクトップアプリ開発時)
- cargo-audit: `cargo install cargo-audit --locked` (一括検証の audit ゲートで必須)

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

push（`master`）と pull request では、GitHub Actions がローカルのゲートと
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

### 一括検証

`./scripts/verify.sh` で、上記ゲート・wasm-test・audit・`trunk build`・パス検査・E2E をまとめて実行できる
（一時出力先は自動で削除される）。全ゲート成功時だけ成果物を残したい場合は
`./scripts/verify.sh --dist /home/koppe/.local/share/openrustbooks-verified/` を指定する。
保持物の確認・SHA256 検証・serve・復旧手順は
[検証済み配布物の取得・保持・復旧](docs/runbook-verified-build.md) を参照。
公開 installer / updater / 署名成果物が無い現状と、導入・更新・ロールバック手順は
[リリース運用](docs/runbook-release.md) を参照。
`trunk build` は `--offline --locked` で実行し、`Cargo.lock` の黙った更新を防ぐ
（[ADR-0237](docs/decisions/0237-ci-gate-consistency.md)）。

### 手動の E2E 検証

Web 版リーダーの表示と移動は、ヘッドレス Chrome を使った
`tools/verify-web-reader.py` で確認できる。Python 3.10 以降（`websockets`）と
Chrome/Chromium が必要で、サンプル EPUB は一時ディレクトリに生成する。

```bash
cd web && trunk build --dist dist
cd .. && python3 tools/verify-web-reader.py --dist web/dist
```

待機は条件待ちを優先し、到達条件のない固定 sleep は据え置いている
（[ADR-0235](docs/decisions/0235-e2e-conditional-wait.md)）。

## ドキュメント

- [要件定義](docs/requirements.md) - ユーザー要件, 未決定事項, 検証条件
- [アーキテクチャ設計](docs/architecture.md) - 構成案, 開発順序, 技術選定
- [検証済み配布物の取得・保持・復旧](docs/runbook-verified-build.md) - 保持・SHA256 検証・serve・復旧手順
- [WebdriverIO 実 WebView E2E](docs/runbook-webdriver.md) - 実 Tauri WebView の settings 検証手順と制約
- [リリース運用](docs/runbook-release.md) - 導入・更新・ロールバック・既知問題と公開作業の未決定
- [ドキュメント運用規約](docs/documentation-operations.md) - OKF・索引・log・ADR・lint の運用
- [0001-use-system-webview](docs/decisions/0001-use-system-webview.md)
  - Tauriの標準的なOS WebView方式を採用する決定と、その影響。
- [0002-ai-markdown-pipeline](docs/decisions/0002-ai-markdown-pipeline.md)
  - AI API + markdownlint (rumdl) の組み合わせで Markdown 品質を保証するパイプラインを共通コアに実装する決定。
- [0003-native-wasm-target-split](docs/decisions/0003-native-wasm-target-split.md)
  - ネイティブ専用依存をtarget別に分離し、共通コアとWebアプリをwasm32-unknown-unknownでビルド可能にする決定。
- [0004-zip-archive-reader](docs/decisions/0004-zip-archive-reader.md)
  - アーカイブ閲覧の境界、zipクレートの選定理由、パス逸脱と過大展開への対策を記録する。
- [0005-epub-body-reading](docs/decisions/0005-epub-body-reading.md)
  - container.xmlとOPFの解析範囲、quick-xml 0.42の選定理由、本文抽出とパス逸脱への対策を記録する。
- [0006-epub-table-of-contents](docs/decisions/0006-epub-table-of-contents.md)
  - EPUB3 nav と EPUB2 NCX の読み分け、平坦リスト採用の理由、欠落と破損の扱いを記録する。
- [0007-epub-toc-fragments](docs/decisions/0007-epub-toc-fragments.md)
  - 目次エントリに文書内アンカーを第2フィールドとして保持し、空パスのhrefを読み飛ばす決定。
- [0008-epub-cover-and-chapter-title](docs/decisions/0008-epub-cover-and-chapter-title.md)
  - 表紙の宣言の解決順と、目次を優先する章タイトルの解決、欠落と破損の扱いを記録する。
- [0009-epub-landmarks-and-page-list](docs/decisions/0009-epub-landmarks-and-page-list.md)
  - EPUB3 nav の landmarks と page-list を既存の目次項目型で返す決定を記録する。
- [0010-epub-wasm-bindings](docs/decisions/0010-epub-wasm-bindings.md)
  - WasmEpubBook で EPUB の読み取り専用 API を JavaScript から利用する方針を記録する。
- [0011-web-epub-reader](docs/decisions/0011-web-epub-reader.md)
  - WasmEpubBook を使ったブラウザ内 EPUB 読書の初期 UI と安全な表示境界を記録する。
- [0012-epub-html-display](docs/decisions/0012-epub-html-display.md)
  - 章 XHTML の許可リスト方式サニタイズ、画像の Blob URL 化、アンカー移動の決定を記録する。
- [0013-epub-display-hardening](docs/decisions/0013-epub-display-hardening.md)
  - 画像参照の URL 正規化、外部リンクの opener 分離、EPUB サイズ上限の決定を記録する。
- [0014-relative-chapter-links](docs/decisions/0014-relative-chapter-links.md)
  - 相対リンクの解決を core に置き、Web の本文内リンクから章間移動とアンカー移動を行う決定を記録する。
- [0015-reading-position](docs/decisions/0015-reading-position.md)
  - 本ごとの章とフラグメントを localStorage に保存し、開き直したときに復元する決定を記録する。
- [0016-scroll-position](docs/decisions/0016-scroll-position.md)
  - スクロール位置の保存（デバウンス）と描画後の復元（再試行付き）の決定を記録する。
- [0017-navigation-history](docs/decisions/0017-navigation-history.md)
  - 読書中の章移動をメモリ上の履歴に積み、戻る・進むでスクロール位置ごと復元する決定を記録する。
- [0018-reader-display-settings](docs/decisions/0018-reader-display-settings.md)
  - 文字サイズ・行間・ダーク表示の保存と適用、保存した読書位置の削除の決定を記録する。
- [0019-position-retention](docs/decisions/0019-position-retention.md)
  - 保存時刻の記録、90 日の寿命、40 件の上限、容量超過時の縮退の決定を記録する。
- [0020-layout-shift-correction](docs/decisions/0020-layout-shift-correction.md)
  - 画像が確定するまで位置の適用を待つ決定と、フラグメント保存の欠落修正を記録する。
- [0021-keyboard-and-aria](docs/decisions/0021-keyboard-and-aria.md)
  - リーダーのキーボードショートカットとアクセシビリティ属性の決定を記録する。
- [0022-position-export-import](docs/decisions/0022-position-export-import.md)
  - 保存した読書位置を JSON で書き出し・読み込みする決定を記録する。
- [0023-app-wide-theme](docs/decisions/0023-app-wide-theme.md)
  - リーダーのダーク表示トグルをアプリ全体（ツールバー・一覧画面）へ拡張する決定を記録する。
- [0024-history-persistence](docs/decisions/0024-history-persistence.md)
  - 移動履歴を localStorage に保存し、同じ本を開き直したときにも戻る・進むができるようにする決定を記録する。
- [0025-touch-gestures](docs/decisions/0025-touch-gestures.md)
  - 本文領域のスワイプで章を移動する決定を記録する。
- [0026-settings-export-import](docs/decisions/0026-settings-export-import.md)
  - エクスポート・インポート形式を拡張し、表示設定も含める決定を記録する。
- [0027-landmarks-ui](docs/decisions/0027-landmarks-ui.md)
  - EPUB3 nav の landmarks をドロップダウンで表示し、移動できるようにする決定を記録する。
- [0028-page-list-navigation](docs/decisions/0028-page-list-navigation.md)
  - EPUB3 nav の page-list を使い、ページ番号入力で章へ移動できるようにする決定を記録する。
- [0029-history-list-ui](docs/decisions/0029-history-list-ui.md)
  - 移動履歴をドロップダウンで一覧表示し、任意の履歴エントリへ直接ジャンプできるようにする決定を記録する。
- [0030-history-export-import](docs/decisions/0030-history-export-import.md)
  - エクスポート・インポート形式に移動履歴を含める決定を記録する。
- [0031-history-clear](docs/decisions/0031-history-clear.md)
  - 移動履歴を全消去できるようにする決定を記録する。
- [0032-import-merge](docs/decisions/0032-import-merge.md)
  - インポートする読書位置を savedAt で比較し、新しい方を採用する決定を記録する。
- [0033-prefers-color-scheme](docs/decisions/0033-prefers-color-scheme.md)
  - prefers-color-scheme を検知してダーク表示を自動で切り替える決定を記録する。
- [0034-async-image-decoding](docs/decisions/0034-async-image-decoding.md)
  - 本文画像に decoding=\"async\" を付与し、デコードを非同期化して描画をブロックしない決定を記録する。
- [0035-history-delete-entry](docs/decisions/0035-history-delete-entry.md)
  - 選択中の履歴エントリだけを削除できるようにする決定を記録する。
- [0036-export-versioning](docs/decisions/0036-export-versioning.md)
  - エクスポート JSON に形式バージョンを付与し、未知のバージョンを fail-closed で拒否する決定を記録する。
- [0037-scroll-order-test-stability](docs/decisions/0037-scroll-order-test-stability.md)
  - 画像位置復元の E2E がブラウザ自身のスクロール調整を誤検知しないようにする決定を記録する。
- [0038-initial-theme-follow](docs/decisions/0038-initial-theme-follow.md)
  - 設定キーが無い初回起動で OS がダークなら追従を有効にする決定を記録する。
- [0039-forced-colors](docs/decisions/0039-forced-colors.md)
  - Windows のハイコントラスト（Forced Colors Mode）でリーダーの視認性を確保する決定を記録する。
- [0040-delete-confirmation-dialog](docs/decisions/0040-delete-confirmation-dialog.md)
  - 履歴の削除操作にネイティブ &lt;dialog&gt; の確認を挟み、誤操作を防ぐ決定を記録する。
- [0041-history-summary](docs/decisions/0041-history-summary.md)
  - 履歴エントリに保存日時を記録し、一覧ラベルに表示する決定を記録する。
- [0042-wasm-clippy-cleanup](docs/decisions/0042-wasm-clippy-cleanup.md)
  - wasm32 ターゲットの clippy 警告をゼロにし、コードをクリーンに保つ決定を記録する。
- [0043-svg-blob-sanitization](docs/decisions/0043-svg-blob-sanitization.md)
  - SVG 画像を Blob URL 化する前にスクリプト実行経路を除去する決定を記録する。
- [0044-svg-external-references](docs/decisions/0044-svg-external-references.md)
  - SVG 内の外部スキーム参照を除去し、Blob URL 直接オープン時のプライバシー漏洩を防ぐ決定を記録する。
- [0045-svg-xml-processing-instructions](docs/decisions/0045-svg-xml-processing-instructions.md)
  - SVG から XML 処理命令を除去し、外部 CSS 参照による情報漏洩を防ぐ決定を記録する。
- [0046-fragment-scroll-stability](docs/decisions/0046-fragment-scroll-stability.md)
  - 画像レイアウト未確定によるフラグメント移動のスクロール失敗を解消する決定を記録する。
- [0047-dialog-esc-test](docs/decisions/0047-dialog-esc-test.md)
  - 削除確認ダイアログの Esc キーによるキャンセル動作を E2E で検証する決定を記録する。
- [0048-dialog-focus-test](docs/decisions/0048-dialog-focus-test.md)
  - 削除確認ダイアログで安全な操作（Cancel）にフォーカスが当たることを E2E で検証する決定を記録する。
- [0049-export-import-roundtrip](docs/decisions/0049-export-import-roundtrip.md)
  - 書き出した JSON をそのまま読み込んで位置・設定・履歴が復元されることを E2E で検証する決定を記録する。
- [0050-deep-nesting-safety](docs/decisions/0050-deep-nesting-safety.md)
  - 深い入れ子の XHTML がサニタイズ時に異常終了しないことをテストで確認する決定を記録する。
- [0051-owned-epub-open](docs/decisions/0051-owned-epub-open.md)
  - wasm で EPUB を開くときのアーカイブバイト列の二重コピーを解消する決定を記録する。
- [0052-fragment-retry-hardening](docs/decisions/0052-fragment-retry-hardening.md)
  - フラグメントスクロールの retry を強化し、残るフレークの調査結果を記録する。
- [0053-scroll-reset-root-cause](docs/decisions/0053-scroll-reset-root-cause.md)
  - 連続した章移動で古い位置適用コールバックが新しい章をリセットする問題を世代号で解消する決定を記録する。
- [0054-early-generation-check](docs/decisions/0054-early-generation-check.md)
  - 古い章の画像待ちループを世代チェックで早期停止する決定を記録する。
- [0055-same-chapter-fragment-history](docs/decisions/0055-same-chapter-fragment-history.md)
  - 本文内のアンカー・脚注リンクによる同一章内の移動も履歴に記録する決定を記録する。
- [0056-swipe-threshold-setting](docs/decisions/0056-swipe-threshold-setting.md)
  - 章移動のスワイプ閾値を表示設定から変更できるようにする決定を記録する。
- [0057-content-language](docs/decisions/0057-content-language.md)
  - EPUB の言語を本文要素の lang 属性に反映し、スクリーンリーダーの発音を最適化する決定を記録する。
- [0058-region-aria-labelledby](docs/decisions/0058-region-aria-labelledby.md)
  - 本文領域の aria-labelledby を章タイトルに結びつける決定を記録する。
- [0059-malformed-xhtml-policy](docs/decisions/0059-malformed-xhtml-policy.md)
  - OPF は fail-closed、章本文はサニタイズに委譲する実際の挙動を確認し記録する。
- [0060-remote-resources-policy](docs/decisions/0060-remote-resources-policy.md)
  - remote-resources を宣言する EPUB を受け付け、リモートアクセスは表示側の防御に委ねる決定を記録する。
- [0061-vertical-scroll-threshold](docs/decisions/0061-vertical-scroll-threshold.md)
  - タッチ操作の縦スクロール判定閾値を設定から変更できるようにする決定を記録する。
- [0062-browser-pinch-zoom](docs/decisions/0062-browser-pinch-zoom.md)
  - リーダーがブラウザ標準のピンチズームを阻害していないことを確認する決定を記録する。
- [0063-legacy-roundtrip](docs/decisions/0063-legacy-roundtrip.md)
  - 旧形式 JSON のインポートからエクスポート・再インポートまでの連鎖を検証する決定を記録する。
- [0064-large-image-warning](docs/decisions/0064-large-image-warning.md)
  - 本文の巨大画像を検出し、状態メッセージで警告する決定を記録する。
- [0065-deep-nesting-10k](docs/decisions/0065-deep-nesting-10k.md)
  - より深い入れ子 XHTML の挙動を回帰テストとして固定する決定を記録する。
- [0066-remote-resources-warning](docs/decisions/0066-remote-resources-warning.md)
  - remote-resources を宣言する EPUB を開いたとき、利用者に警告を表示する決定を記録する。
- [0067-optional-image-downscale](docs/decisions/0067-optional-image-downscale.md)
  - 巨大画像を Canvas で縮小する設定を既定オフで追加する決定を記録する。
- [0068-reset-settings](docs/decisions/0068-reset-settings.md)
  - 増えた表示設定をまとめて既定値に戻すボタンを追加する決定を記録する。
- [0069-timestamped-export-filename](docs/decisions/0069-timestamped-export-filename.md)
  - エクスポート JSON のダウンロード名にローカル時刻のタイムスタンプを付ける決定を記録する。
- [0070-export-filename-timezone](docs/decisions/0070-export-filename-timezone.md)
  - エクスポート JSON のファイル名にタイムゾーン偏移を付ける決定を記録する。
- [0071-downscale-mime-preservation](docs/decisions/0071-downscale-mime-preservation.md)
  - 画像縮小時に元画像の MIME タイプを保持する決定を記録する。
- [0072-downscale-jpeg-quality](docs/decisions/0072-downscale-jpeg-quality.md)
  - 画像縮小時の JPEG 再エンコードに品質 0.85 を明示する決定を記録する。
- [0073-jpeg-quality-setting](docs/decisions/0073-jpeg-quality-setting.md)
  - 縮小 JPEG の品質を表示設定から変更できるようにする決定を記録する。
- [0074-jpeg-size-comparison](docs/decisions/0074-jpeg-size-comparison.md)
  - 縮小 JPEG の品質差がファイルサイズに表れない場合があることを調査し記録する。
- [0075-shortcuts-help](docs/decisions/0075-shortcuts-help.md)
  - ? キーとボタンでショートカット一覧を表示する決定を記録する。
- [0076-version-migration-policy](docs/decisions/0076-version-migration-policy.md)
  - 形式バージョン 2 以降を導入するときの受け付け範囲・デフォルト値・移行手順を定義する。
- [0077-version-1-sufficiency](docs/decisions/0077-version-1-sufficiency.md)
  - 現時点では version 1 で十分であることをレビューし、version 2 の導入条件を将来の候補として記録する。
- [0078-chapter-filter](docs/decisions/0078-chapter-filter.md)
  - 章ドロップダウンにタイトル検索フィルターを追加する決定を記録する。
- [0079-filter-focus-shortcut](docs/decisions/0079-filter-focus-shortcut.md)
  - / キーで章フィルター入力にフォーカスする決定を記録する。
- [0080-external-link-safety-test](docs/decisions/0080-external-link-safety-test.md)
  - 本文内の外部リンクが noopener noreferrer 付きで保持されることを E2E で検証する決定を記録する。
- [0081-external-link-popup-test](docs/decisions/0081-external-link-popup-test.md)
  - 外部リンクのクリックが noopener/noreferrer 付きの window.open 呼び出しになることを E2E で検証する決定を記録する。
- [0082-mailto-link-test](docs/decisions/0082-mailto-link-test.md)
  - mailto: リンクが外部リンクと同じ安全な分岐で開かれることを E2E で検証する決定を記録する。
- [0083-dead-code-cleanup](docs/decisions/0083-dead-code-cleanup.md)
  - 調査で検出された未使用関数と古い E2E 項目数を整理する決定を記録する。
- [0084-verification-script](docs/decisions/0084-verification-script.md)
  - 全ゲート・Wasm ビルド・E2E を再現可能に実行するスクリプトを追加する決定を記録する。
- [0085-single-entry-history-save](docs/decisions/0085-single-entry-history-save.md)
  - 履歴が 1 件になっても localStorage を更新する決定を記録する。
- [0086-dependency-vulnerability-fix](docs/decisions/0086-dependency-vulnerability-fix.md)
  - ammonia と pulldown-cmark の脆弱性を修正する決定を記録する。
- [0087-front-matter-preservation](docs/decisions/0087-front-matter-preservation.md)
  - Markdown 文書の保存時に front matter が失われるバグを修正する決定を記録する。
- [0088-wasm-path-stripping](docs/decisions/0088-wasm-path-stripping.md)
  - 配布 WASM に含まれる開発ホスト絶対パスを除去する決定を記録する。
- [0089-init-unification](docs/decisions/0089-init-unification.md)
  - Rust / JS 双方の init() が同じ初期化処理を呼ぶようにする決定を記録する。
- [0090-e2e-expected-count](docs/decisions/0090-e2e-expected-count.md)
  - 未実行の check を見逃さないため、期待総数を宣言して検証する決定を記録する。
- [0091-content-security-policy](docs/decisions/0091-content-security-policy.md)
  - Web 版に CSP meta を追加し、リソース読み込み先を制限する決定を記録する。
- [0092-ime-composing-guard](docs/decisions/0092-ime-composing-guard.md)
  - IME の変換中はキーボードショートカットを発火させない決定を記録する。
- [0093-modal-shortcut-guard](docs/decisions/0093-modal-shortcut-guard.md)
  - dialog が開いているときは背景のキーボードショートカットを止める決定を記録する。
- [0094-desktop-window-result](docs/decisions/0094-desktop-window-result.md)
  - get_webview_window の unwrap() を Result 化し、起動時の panic を防ぐ決定を記録する。
- [0095-empty-export-and-text-shrink](docs/decisions/0095-empty-export-and-text-shrink.md)
  - 保存データ 0 件のエクスポート経路と文字縮小操作を E2E で検証し、空ファイル生成を防ぐ決定を記録する。
- [0096-appstate-tests](docs/decisions/0096-appstate-tests.md)
  - desktop の AppState に Tauri ランタイム不要の状態遷移テストを追加する決定を記録する。
- [0097-path-containment](docs/decisions/0097-path-containment.md)
  - LocalStorage::resolve_path を fail-closed 化し、root 外へのパス脱出を拒否する決定を記録する。
- [0098-secret-exposure-mitigation](docs/decisions/0098-secret-exposure-mitigation.md)
  - API キー等の IPC 露出をマスクし、Debug とファイル権限を硬化する決定を記録する。
- [0099-ai-default-off](docs/decisions/0099-ai-default-off.md)
  - AI 機能を既定で無効にし、明示的な opt-in まで本文を送信しないようにする決定を記録する。
- [0100-ai-service-guard](docs/decisions/0100-ai-service-guard.md)
  - AI を無効化した後に service が残存して送信できる問題を修正する決定を記録する。
- [0101-base-url-allowlist](docs/decisions/0101-base-url-allowlist.md)
  - AI の送信先 URL を既定ホストと loopback のみに制限し、JS からの任意送信先差し替えを防ぐ決定を記録する。
- [0102-extra-headers-redaction](docs/decisions/0102-extra-headers-redaction.md)
  - OpenAiConfig.extra_headers を IPC DTO と Debug 出力からマスクする決定を記録する。
- [0103-path-field-redaction](docs/decisions/0103-path-field-redaction.md)
  - 設定 IPC の DTO からローカルパス情報を除外し、マージで保持する決定を記録する。
- [0104-send-consent-dialog](docs/decisions/0104-send-consent-dialog.md)
  - 本文を AI へ送る直前に Rust 側でネイティブ確認を要求する決定を記録する。
- [0105-retry-config](docs/decisions/0105-retry-config.md)
  - リトライ回数をトークン上限から分離し、AiConfig.max_retries に集約する決定を記録する。
- [0106-retry-merge-preservation](docs/decisions/0106-retry-merge-preservation.md)
  - max_retries を DTO から除外し、IPC 往復で現在値を保持する決定を記録する。
- [0107-ai-service-refresh](docs/decisions/0107-ai-service-refresh.md)
  - AI/lint 設定変更時に service を fail-closed で再初期化する決定を記録する。
- [0108-prompt-system-prompt](docs/decisions/0108-prompt-system-prompt.md)
  - 内蔵 .hbs テンプレートに必須フィールドを追加し、AI サービスの初期化失敗を解消する決定を記録する。
- [0109-system-prompt-unification](docs/decisions/0109-system-prompt-unification.md)
  - OpenAI の固定 system message を削除し、system 指示をプロンプトテンプレートに一元化する決定を記録する。
- [0110-lint-auto-fix](docs/decisions/0110-lint-auto-fix.md)
  - rumdl の fix API を用いて Markdown lint の自動修正を実装し、auto_fix 設定を配線する決定を記録する。
- [0111-fix-convergence](docs/decisions/0111-fix-convergence.md)
  - rumdl fix の未収束情報を保持し、未収束時は修正を採用しない決定を記録する。
- [0112-lazy-image-loading](docs/decisions/0112-lazy-image-loading.md)
  - IntersectionObserver による画像遅延読み込みは全面切替を見送り、寸法予約→段階導入の順で検討する決定を記録する。
- [0113-image-settle-hardening](docs/decisions/0113-image-settle-hardening.md)
  - images_settled() に自然幅の確認を追加し、complete() だけでは見逃す画像を待つ決定を記録する。
- [0114-expected-image-count](docs/decisions/0114-expected-image-count.md)
  - expected_images を src を持つ画像の数にし、src 削除画像による無駄な 2 秒待ちを解消する決定を記録する。
- [0115-epub-css-policy](docs/decisions/0115-epub-css-policy.md)
  - EPUB 内 CSS は現状どおり適用しないことを維持し、将来のオプトイン・スコープ付き段階導入の方針を記録する。
- [0116-final-lint-policy](docs/decisions/0116-final-lint-policy.md)
  - AI pipeline の最終試行で lint 不合格でも Ok を維持し、呼び出し側が lint_passed を確認して差分を提示する方針を記録する。
- [0117-wasm-config-alignment](docs/decisions/0117-wasm-config-alignment.md)
  - WasmConfig の ai_enabled 既定値を false にし、native の AiConfig と揃える決定を記録する。
- [0118-pipeline-dead-code](docs/decisions/0118-pipeline-dead-code.md)
  - execute_pipeline の到達不能な last_error と動的 Err を削除する決定を記録する。
- [0119-wasm-test-runner](docs/decisions/0119-wasm-test-runner.md)
  - wasm-bindgen-test と wasm-pack により wasm32 単体テストを実際に実行するゲートを追加する決定を記録する。
- [0120-retry-limit](docs/decisions/0120-retry-limit.md)
  - max_retries を service 境界で 0..=10 にクランプし、暴走とオーバーフローを防ぐ決定を記録する。
- [0121-req010-retry-wording](docs/decisions/0121-req010-retry-wording.md)
  - REQ-010 の「最大3回再生成」を実装と一致する文言に明文化する決定を記録する。
- [0122-env-var-mapping](docs/decisions/0122-env-var-mapping.md)
  - figment の split('_') で届かない max_retries を extract 後に手動で読み込む決定を記録する。
- [0123-warnings-as-errors](docs/decisions/0123-warnings-as-errors.md)
  - LintConfig.warnings_as_errors を lint 判定に反映し、Warning をオプトインで不合格扱いにする決定を記録する。
- [0124-rule-config-wiring](docs/decisions/0124-rule-config-wiring.md)
  - rule_config を rumdl Config に反映し、config_file は extends のリスクから未対応とする決定を記録する。
- [0125-audit-gate](docs/decisions/0125-audit-gate.md)
  - cargo audit をローカル検証スクリプトに追加し、既知脆弱性の回帰を防ぐ決定を記録する。
- [0126-ci-audit-gate](docs/decisions/0126-ci-audit-gate.md)
  - 共通ワークフローの audit 入力を有効化し、CI でも cargo audit を実行する決定を記録する。
- [0127-editor-ui-integration](docs/decisions/0127-editor-ui-integration.md)
  - CodeMirror を導入せず、既存の Tauri command を段階的に Web UI へ接続する方針を記録する。
- [0128-settings-form-wiring](docs/decisions/0128-settings-form-wiring.md)
  - SettingsForm に Tauri IPC の読み取り表示とブラウザ版 fallback を追加し、保存接続は安全性の観点から見送る決定を記録する。
- [0129-document-path-validation](docs/decisions/0129-document-path-validation.md)
  - desktop の document command で AppConfig.local_roots によるパス検証を行う決定を記録する。
- [0130-desktop-csp](docs/decisions/0130-desktop-csp.md)
  - desktop/tauri.conf.json の script-src から unsafe-eval を除外し wasm-unsafe-eval に置換する決定を記録する。
- [0131-config-patch-command](docs/decisions/0131-config-patch-command.md)
  - ConfigView と同型の ConfigPatch を受け取る update_config_patch を追加し、Web UI からの安全な設定保存経路を準備する決定を記録する。
- [0132-preserve-logic-unification](docs/decisions/0132-preserve-logic-unification.md)
  - ConfigPatch::apply_to を公開フィールドの上書きのみとし、隠しフィールドの保持を merge_secrets に一本化する決定を記録する。
- [0133-zero-retry-regression](docs/decisions/0133-zero-retry-regression.md)
  - 初期1回のみ実行される挙動をテストで固定し、ADR-0118/0120 の未決定を解消する。
- [0134-secret-removal-command](docs/decisions/0134-secret-removal-command.md)
  - reset_config を使わずに API キー等だけを削除する clear_secrets コマンドを追加する決定を記録する。
- [0135-secret-clear-audit-log](docs/decisions/0135-secret-clear-audit-log.md)
  - clear_secrets が削除対象名だけをログに残す決定を記録する。
- [0136-image-dimension-reservation](docs/decisions/0136-image-dimension-reservation.md)
  - 画像の自然寸法を width/height 属性で予約する段階導入の設計方針を記録する。
- [0137-imagesize-wasm-compat](docs/decisions/0137-imagesize-wasm-compat.md)
  - 画像寸法取得クレート imagesize が wasm32 で使えることを隔離検証で確認し、段階導入の前提を解消する。
- [0138-position-e2e-baseline](docs/decisions/0138-position-e2e-baseline.md)
  - 画像寸法予約の実装前に位置復元関連 E2E を 5 回連続実行し、現状のベースラインとフレークを記録する。
- [0139-image-warning-flake](docs/decisions/0139-image-warning-flake.md)
  - 位置適用の 2 秒 deadline を過ぎて読み込まれる巨大画像を load イベントで検出し、警告の取りこぼしを防ぐ決定を記録する。
- [0140-extended-position-baseline](docs/decisions/0140-extended-position-baseline.md)
  - 画像寸法予約前に位置復元系 E2E を 10 回連続実行し、ベースラインと残るフレークを記録する。
- [0141-downscale-flake](docs/decisions/0141-downscale-flake.md)
  - 画像縮小の完了待ちを条件確認＋再待機にし、E2E の取りこぼしを防ぐ決定を記録する。
- [0142-startup-flake](docs/decisions/0142-startup-flake.md)
  - E2E 起動時に CDP 診断イベントを収集し、マウント失敗時のみ 1 回再試行する決定を記録する。
- [0143-cdp-target-matching](docs/decisions/0143-cdp-target-matching.md)
  - Chrome の browser-level WebSocket を使って E2E が起動した Chrome に確実に接続する決定を記録する。
- [0144-cdp-recv-timeout](docs/decisions/0144-cdp-recv-timeout.md)
  - E2E の CDP 応答待ちにタイムアウトを設け、ハング時に fail-closed で診断を残す決定を記録する。
- [0145-vertical-writing-pagination](docs/decisions/0145-vertical-writing-pagination.md)
  - 縦書き表示とページ送りを即座に実装せず、E2E の軸非依存化から始める段階導入方針を記録する。
- [0146-axis-independent-e2e-scroll](docs/decisions/0146-axis-independent-e2e-scroll.md)
  - E2E の DOM スクロール読み書きを writing-mode に応じて軸を選ぶヘルパーへ集約する決定を記録する。
- [0147-opt-in-vertical-writing-rejected](docs/decisions/0147-opt-in-vertical-writing-rejected.md)
  - 縦書きのアプリ側実装を試行したが、位置保存・復元を検証できなかったため見送った決定を記録する。
- [0148-downscale-completion-signal](docs/decisions/0148-downscale-completion-signal.md)
  - 画像縮小の完了を data-downscaled で明示し、未確定画像のリトライと MIME 属性化で E2E フレークを解消する決定を記録する。
- [0149-image-dimension-reservation](docs/decisions/0149-image-dimension-reservation.md)
  - EPUB 画像の自然寸法を core で取得し、web で width/height 属性として付与する実装を記録する。
- [0150-img-dimension-validation](docs/decisions/0150-img-dimension-validation.md)
  - EPUB HTML の img の width/height を数値検証し、不正値を削除する決定を記録する。
- [0151-remaining-task-discovery](docs/decisions/0151-remaining-task-discovery.md)
  - 5 観点のエージェント探索で発見した残タスクを集約し、次に実行するタスクを記録する。
- [0152-wasm-path-check-failclosed](docs/decisions/0152-wasm-path-check-failclosed.md)
  - 配布 WASM の開発ホスト絶対パス検査を、glob 非マッチと grep エラーでも失敗するようにする決定を記録する。
- [0153-about-page-accuracy](docs/decisions/0153-about-page-accuracy.md)
  - About 画面のライセンスと対応形式を実装事実に合わせる決定を記録する。
- [0154-storage-failure-visibility](docs/decisions/0154-storage-failure-visibility.md)
  - localStorage への保存失敗時に固定文言の status 表示と console.error を出す決定を記録する。
- [0155-rumdl-feature-reduction](docs/decisions/0155-rumdl-feature-reduction.md)
  - native 側 rumdl を featureless にし、LSP / ファイル監視 / 並列処理スタックを依存ツリーから除去する決定を記録する。
- [0156-wasm-stub-failclosed](docs/decisions/0156-wasm-stub-failclosed.md)
  - ブラウザで AI/lint が成功を偽装していた Wasm API を明示的なエラーに変更する決定を記録する。
- [0157-import-preservation](docs/decisions/0157-import-preservation.md)
  - インポートが現在の設定と履歴を破壊しないよう、現在値が存在する場合は維持する決定を記録する。
- [0158-partial-settings-import](docs/decisions/0158-partial-settings-import.md)
  - 部分設定インポートの bool false リセットが現行ガード下では再現しないことを記録する。
- [0159-cls-e2e-measurement](docs/decisions/0159-cls-e2e-measurement.md)
  - PerformanceObserver('layout-shift') で CLS を計測し、画像章オープン時のレイアウトシフトをゲートする決定を記録する。
- [0160-real-wasm-lint](docs/decisions/0160-real-wasm-lint.md)
  - fail-closed スタブだった WasmMarkdownLinter を featureless rumdl の実 lint に接続する決定を記録する。
- [0161-minimal-editor-ui](docs/decisions/0161-minimal-editor-ui.md)
  - CodeMirror を導入せず、textarea と既存 Preview を接続した最小エディタを実装する決定を記録する。
- [0162-markdown-preview-rendering](docs/decisions/0162-markdown-preview-rendering.md)
  - エディタ Preview で comrak による Markdown レンダリングと二段構えのサニタイズを行う決定を記録する。
- [0163-remove-unused-markdown-dep](docs/decisions/0163-remove-unused-markdown-dep.md)
  - 直接未使用となった pulldown-cmark をマニフェストから削除する決定を記録する。
- [0164-editor-draft-save](docs/decisions/0164-editor-draft-save.md)
  - Tauri を有効化せず、エディタの Save をブラウザ内 localStorage ドラフトとして接続する決定を記録する。
- [0165-save-error-listener](docs/decisions/0165-save-error-listener.md)
  - 保存失敗通知を送信先別イベント名へ分離し、Editor / Reader の status 競合を解消する決定を記録する。
- [0166-listener-cleanup](docs/decisions/0166-listener-cleanup.md)
  - forget() によるリスナー蓄積を on_cleanup + SendWrapper による解除へ変更する決定を記録する。
- [0167-draft-size-limit](docs/decisions/0167-draft-size-limit.md)
  - 下書き保存に 1 MiB の UTF-8 バイト上限を設け、超過時は保存せず固定文言で通知する決定を記録する。
- [0168-workspace-rumdl-cleanup](docs/decisions/0168-workspace-rumdl-cleanup.md)
  - 未使用の workspace.dependencies.rumdl を削除し、default features 復活の罠を排除する決定を記録する。
- [0169-partial-storage-failures](docs/decisions/0169-partial-storage-failures.md)
  - 読書位置の全消去・インポートの部分失敗を操作単位で通知する決定を記録する。
- [0170-oversized-draft-restore](docs/decisions/0170-oversized-draft-restore.md)
  - 1 MiB 超の保存済み下書きは本文を保持したまま復元し、固定警告のみを表示する決定を記録する。
- [0171-clear-data-confirmation](docs/decisions/0171-clear-data-confirmation.md)
  - 読書位置の全消去に window.confirm() の確認を追加する決定を記録する。
- [0172-import-confirmation](docs/decisions/0172-import-confirmation.md)
  - バックアップの取り込み前に window.confirm() の確認を求める決定を記録する。
- [0173-overwrite-guard](docs/decisions/0173-overwrite-guard.md)
  - overwrite=false のとき既存ファイルへの書き込みを拒否し、黙って上書きされる問題を解消する決定を記録する。
- [0174-read-text-utf8](docs/decisions/0174-read-text-utf8.md)
  - 不正 UTF-8 の内容をエラー文言へ埋め込まず、U+FFFD 置換で返す決定を記録する。
- [0175-reset-settings-failure](docs/decisions/0175-reset-settings-failure.md)
  - 設定リセットが保存失敗通知を成功文言で上書きしないようにする決定を記録する。
- [0176-epub-load-generation](docs/decisions/0176-epub-load-generation.md)
  - 非同期 EPUB ロードに独立した世代カウンタを導入し、古いロード完了による状態上書きを防ぐ決定を記録する。
- [0177-config-commit-transaction](docs/decisions/0177-config-commit-transaction.md)
  - 設定保存と AI サービス再初期化を準備／コミットに分け、失敗時の部分状態を防ぐ決定を記録する。
- [0178-watch-capability-honesty](docs/decisions/0178-watch-capability-honesty.md)
  - 未実装の watch を利用可能と報告せず、到達不能な IPC も削除する決定を記録する。
- [0179-editor-autosave](docs/decisions/0179-editor-autosave.md)
  - 明示 Save に加えてデバウンス自動保存を導入し、画面遷移時の下書き損失を防ぐ決定を記録する。
- [0180-draft-backup](docs/decisions/0180-draft-backup.md)
  - バックアップ JSON に下書きを optional field として含め、復元時に現在値を保護する決定を記録する。
- [0181-svg-sanitizer-unification](docs/decisions/0181-svg-sanitizer-unification.md)
  - Web 側の DomParser 手動サニタイズを廃止し、core の ammonia ベース実装へ一本化する決定を記録する。
- [0182-e2e-unique-check-names](docs/decisions/0182-e2e-unique-check-names.md)
  - Checker に check 名の重複検知を追加し、総数ゲートの形骸化を防ぐ決定を記録する。
- [0183-wait-until-failclosed](docs/decisions/0183-wait-until-failclosed.md)
  - E2E 待機ヘルパーで握り潰していた Python 例外を再送出する決定を記録する。
- [0184-image-dimension-tests](docs/decisions/0184-image-dimension-tests.md)
  - image_dimensions の JPEG / WebP ヘッダーと境界値テストを追加する決定を記録する。
- [0185-audit-tool-failclosed](docs/decisions/0185-audit-tool-failclosed.md)
  - cargo-audit 未インストール時に検証をスキップせず失敗させる決定を記録する。
- [0186-config-ipc-boundary](docs/decisions/0186-config-ipc-boundary.md)
  - 生 config を受け取るコマンドでも DTO 境界へ正規化し、機微フィールドの注入を防ぐ決定を記録する。
- [0187-table-attribute-validation](docs/decisions/0187-table-attribute-validation.md)
  - EPUB サニタイズで table 関連の数値属性を検証し、不正値を削除する決定を記録する。
- [0188-atomic-tmp-uniqueness](docs/decisions/0188-atomic-tmp-uniqueness.md)
  - atomic 書き込みの tmp 名に PID とカウンタを付与し、並行書き込みの衝突を防ぐ決定を記録する。
- [0189-ollama-stream-boundary](docs/decisions/0189-ollama-stream-boundary.md)
  - Ollama の JSONL ストリームをバッファリングし、HTTP タイムアウトとエラーを fail-closed で扱う決定を記録する。
- [0190-read-limit-hardening](docs/decisions/0190-read-limit-hardening.md)
  - StorageBackend::read の要求サイズに上限を設け、超過時は黙って切り詰めずエラーにする決定を記録する。
- [0191-external-font-removal](docs/decisions/0191-external-font-removal.md)
  - Google Fonts の preconnect / stylesheet を廃止し、CSP とフォントスタックをローカル完結にする決定を記録する。
- [0192-blob-url-cleanup](docs/decisions/0192-blob-url-cleanup.md)
  - Reader アンマウント時に残余の object URL を revoke し、SPA 往復によるメモリ残存を防ぐ決定を記録する。
- [0193-settings-form-readonly](docs/decisions/0193-settings-form-readonly.md)
  - ブラウザ版の設定フォームを契約どおり読み取り専用にし、変更できると誤解させない決定を記録する。
- [0194-lint-validation-unification](docs/decisions/0194-lint-validation-unification.md)
  - native と wasm で重複していた auto_fix 分岐を lint.rs の共通関数へ集約する決定を記録する。
- [0195-rumdl-feature-gate](docs/decisions/0195-rumdl-feature-gate.md)
  - core/Cargo.toml の rumdl default-features=false をローカルと CI の両方で静的検査する決定を記録する。
- [0196-ci-wasm-test-gate](docs/decisions/0196-ci-wasm-test-gate.md)
  - wasm-bindgen-test を CI でも実行するため、共通スクリプトを verify.sh と extra_gates から呼ぶ決定を記録する。
- [0197-dead-trait-methods](docs/decisions/0197-dead-trait-methods.md)
  - TokenStream::is_done と StorageBackend::available_space を削除し、未実装 API への誤依存を防ぐ決定を記録する。
- [0198-reader-a11y-fixes](docs/decisions/0198-reader-a11y-fixes.md)
  - 本文領域のアクセシブル名を aria-labelledby へ一本化し、ダーク切替の状態を実効値で告知する決定を記録する。
- [0199-third-party-notices](docs/decisions/0199-third-party-notices.md)
  - 配布に関係する依存クレートを cargo metadata から網羅し、ライセンス ID と出典を第三者通知へ記載する決定を記録する。
- [0200-send-wrapper-direct-dep](docs/decisions/0200-send-wrapper-direct-dep.md)
  - Leptos の doc(hidden) re-export を避け、send_wrapper を web クレートの直接依存として使う決定を記録する。
- [0201-build-app-plugin-dedup](docs/decisions/0201-build-app-plugin-dedup.md)
  - desktop / mobile の build_app で重複していた共通プラグイン設定を 1 関数へ集約する決定を記録する。
- [0202-ai-command-error-tests](docs/decisions/0202-ai-command-error-tests.md)
  - confirmation / format / alignment の分岐を純粋関数へ抽出し、既存挙動を unit test で固定する決定を記録する。
- [0203-cover-image-dto](docs/decisions/0203-cover-image-dto.md)
  - Web 側の CoverImage を受信に必要な 2 フィールドへ縮小し、allow(dead_code) を削除する決定を記録する。
- [0204-env-mapping](docs/decisions/0204-env-mapping.md)
  - figment の split 依存をやめ、明示マップと生値検証で ADR-0122 の契約を維持する決定を記録する。
- [0205-wasm-config-removal](docs/decisions/0205-wasm-config-removal.md)
  - 未使用の WasmConfig 公開 API を削除し、設定モデル重複の 1 つを解消する決定を記録する。
- [0206-e2e-traceability](docs/decisions/0206-e2e-traceability.md)
  - 要件の検証条件に VC ID を付与し、E2E check 名との対応表を追加する決定を記録する。
- [0207-appstate-scope](docs/decisions/0207-appstate-scope.md)
  - 設定系トランザクションの完了を確認し、AppState を composition root として設計受容する決定を記録する。
- [0208-unsaved-changes-cleanup](docs/decisions/0208-unsaved-changes-cleanup.md)
  - 文書削除時に unsaved_changes のキーも削除し、force close 後の終了確認誤作動を防ぐ決定を記録する。
- [0209-atomic-config-save](docs/decisions/0209-atomic-config-save.md)
  - config.toml を tmp ファイル＋rename で置換し、保存途中失敗による部分破壊を防ぐ決定を記録する。
- [0210-config-revision-cas](docs/decisions/0210-config-revision-cas.md)
  - 生成失敗待ち中の設定更新を CAS で検出し、lost update を防ぐ決定を記録する。
- [0211-config-layer-boundaries](docs/decisions/0211-config-layer-boundaries.md)
  - 3 設定レイヤーの権威・寿命分離を設計受容し、統合を却下する決定。
- [0212-status-error-redaction](docs/decisions/0212-status-error-redaction.md)
  - 内部エラーの Debug 表現を status と console から排除し、操作別の固定文言のみを表示する決定を記録する。
- [0213-reduced-motion](docs/decisions/0213-reduced-motion.md)
  - OS の reduce 設定で空間移動を抑制し、色 transition と位置復元の正確性を維持する決定を記録する。
- [0214-msrv](docs/decisions/0214-msrv.md)
  - 依存クレートの実際の要件に合わせて Rust 1.95 を MSRV とし、CI でも同じツールチェーンを使う決定を記録する。
- [0215-dependabot](docs/decisions/0215-dependabot.md)
  - Cargo と GitHub Actions の patch / minor 更新を週次で自動検出し、major は手動判断に残す決定を記録する。
- [0216-streaming-deferred](docs/decisions/0216-streaming-deferred.md)
  - ストリーミング API の本番接続を見送り、設定表示を未配線として正直化する決定を記録する。
- [0217-editor-reader-dead-config](docs/decisions/0217-editor-reader-dead-config.md)
  - config.toml の Editor / Reader 値を DTO に残しつつ、動作に未接続であることを設定画面へ明記する決定を記録する。
- [0218-custom-ai-endpoint](docs/decisions/0218-custom-ai-endpoint.md)
  - カスタム AI 送信先の opt-in を見送り、allowlist の現状契約と将来の最小安全設計を記録する。
- [0219-verified-artifact-retention](docs/decisions/0219-verified-artifact-retention.md)
  - verify.sh --dist で全ゲート成功時のみ dist をリポジトリ外へ保持し、manifest で完全性を記録する決定。
- [0220-traceability-refresh](docs/decisions/0220-traceability-refresh.md)
  - ADR-0206 の追従漏れを解消し、253 件の E2E check 名を VC 対応表へ完全一致で反映する決定。
- [0221-ai-client-options](docs/decisions/0221-ai-client-options.md)
  - OpenAI extra_headers と Ollama options を安全にクライアントへ渡す決定を記録する。
- [0222-updater-deferred](docs/decisions/0222-updater-deferred.md)
  - tauri_plugin_updater の初期化のみで更新チェックは未配線とし、設定表示を「未配線」へ正直化する決定を記録する。
- [0223-zip-wasm-api](docs/decisions/0223-zip-wasm-api.md)
  - fail-closed な ZIP core を wasm32 向け API として公開し、UI は後段へ分離する決定を記録する。
- [0224-todo-reconciliation](docs/decisions/0224-todo-reconciliation.md)
  - 監査で解消済みと判定した TODO を ADR 実績へ照合し、残リスクを ADR の未決定へ集約する決定を記録する。
- [0225-broken-image-settling](docs/decisions/0225-broken-image-settling.md)
  - 画像の error を記録して失敗画像を待ち対象外にし、章ごとの 2 秒待ちを防ぐ決定を記録する。
- [0226-status-message-language](docs/decisions/0226-status-message-language.md)
  - lang=\"ja\" 固定の Web UI で Reader status を日本語へ統一し、aria-live 分離は後段へ残す決定を記録する。
- [0227-ai-client-injection](docs/decisions/0227-ai-client-injection.md)
  - core 内のモック注入を維持し、公開注入点を追加しない決定と将来の最小設計を記録する。
- [0228-ai-dead-config](docs/decisions/0228-ai-dead-config.md)
  - 月間上限の未配線と timeout の Ollama 限定適用を設定画面へ明示し、実装を見送る決定を記録する。
- [0229-error-message-cleanup](docs/decisions/0229-error-message-cleanup.md)
  - 二重 prefix と JsValue Debug の露出を解消し、desktop の panic を Result ベースのエラー表示へ変更する決定を記録する。
- [0230-dark-button-contrast](docs/decisions/0230-dark-button-contrast.md)
  - dark のプライマリボタン文字色を暗色トークンへ変更し、WCAG AA を満たす決定を記録する。
- [0231-unknown-backend-tests](docs/decisions/0231-unknown-backend-tests.md)
  - 未検出ストレージバックエンドと未知 AI provider のエラー分岐を unit test で固定する決定を記録する。
- [0232-initial-release-scope](docs/decisions/0232-initial-release-scope.md)
  - 要件と実装の境界を実証付きで明示し、初版で提供する範囲と未決定を切り分ける決定を記録する。
- [0233-solution-selection-rationale](docs/decisions/0233-solution-selection-rationale.md)
  - 主要依存のライセンス・鮮度・既知脆弱性・Wasm 対応を横断比較し、選定根拠を一箇所に記録する決定。
- [0234-documentation-operations](docs/decisions/0234-documentation-operations.md)
  - OKF・索引・log・lint の規約と設定をリポジトリ内へ集約する決定。
- [0235-e2e-conditional-wait](docs/decisions/0235-e2e-conditional-wait.md)
  - swipe の二重 dispatch を 1 回へし、到達条件が明確な待機を条件待ちへ置換する決定。
- [0236-prefers-contrast](docs/decisions/0236-prefers-contrast.md)
  - prefers-contrast: more で枠線と補助文字を強化し、forced-colors の既存契約を後方で維持する決定。
- [0237-ci-gate-consistency](docs/decisions/0237-ci-gate-consistency.md)
  - wasm_targets と extra_gates の役割分担を確認し、verify.sh の trunk build に --locked を付けて Cargo.lock の不変を保証する決定。
- [0238-deterministic-compression-tests](docs/decisions/0238-deterministic-compression-tests.md)
  - ZIP エントリの方式とメタデータを直接検証し、deflate の圧縮率に依存するテストを排除する決定を記録する。
- [0239-version-single-source](docs/decisions/0239-version-single-source.md)
  - tauri.conf.json の version を削除し、Cargo workspace version へのフォールバックと静的ゲートで drift を防ぐ決定を記録する。
- [0240-duplicate-archive-entries](docs/decisions/0240-duplicate-archive-entries.md)
  - parser differential を防ぐため、正規化後に同名となる ZIP エントリを含むアーカイブを fail-closed で拒否する決定を記録する。
- [0241-generated-timestamp-semantics](docs/decisions/0241-generated-timestamp-semantics.md)
  - ADR の generated.at は生成・正規化時点の記録であり作成時刻ではないと明記し、既存メタデータを保持する決定。
- [0242-rumdl-feature-status](docs/decisions/0242-rumdl-feature-status.md)
  - native / wasm とも rumdl featureless である現状を正とし、ADR-0003 の target 別 feature 選択を履歴として扱う決定を記録する。
- [0243-unused-wasm-api-removal](docs/decisions/0243-unused-wasm-api-removal.md)
  - wasm_bindgen の JS export を属性ごと削除し、未使用の greet / detect_format を公開面から外す決定を記録する。
- [0244-wasm-document-removal](docs/decisions/0244-wasm-document-removal.md)
  - 未使用の WasmDocument export を削除し、Wasm API の第二段を完了する決定を記録する。
- [0245-position-prune-notification](docs/decisions/0245-position-prune-notification.md)
  - localStorage 容量超過で古い読書位置を削除した場合に件数と残件数を通知する決定を記録する。
- [0246-workspace-dependency-unification](docs/decisions/0246-workspace-dependency-unification.md)
  - feature 差分が無い core 依存を workspace.dependencies 参照へ統一する決定を記録する。
- [0247-epub-owned-limits](docs/decisions/0247-epub-owned-limits.md)
  - EpubBook の所有権移譲 API に ArchiveLimits を指定可能にし、借用 API と同じ検証契約を共有する決定を記録する。
- [0248-epub-manifest-errors](docs/decisions/0248-epub-manifest-errors.md)
  - manifest item の id / href 欠落を固定文言の Archive エラーとして fail-closed に報告する決定を記録する。
- [0249-cover-media-type](docs/decisions/0249-cover-media-type.md)
  - 未宣言の media-type を許容し明示された非画像を拒否する ADR-0008 の契約を unit test で固定する決定を記録する。
- [0250-document-epub-type-removal](docs/decisions/0250-document-epub-type-removal.md)
  - DocumentContent::Epub と重複する 3 構造体を削除し、EPUB は EpubBook 経路へ一本化する決定を記録する。
- [0251-expanded-size-budget](docs/decisions/0251-expanded-size-budget.md)
  - ZIP エントリの実展開バイトを AtomicU64 で累計し、宣言サイズ合計では防げない反復読み出しを制限する決定を記録する。
- [0252-path-depth-limit](docs/decisions/0252-path-depth-limit.md)
  - normalize_path と resolve_reference のコンポーネント数が無制限であることを、非再帰性と既存防御の責任分界から受容する決定を記録する。
- [0253-tauri-settings-save-design](docs/decisions/0253-tauri-settings-save-design.md)
  - browser の読み取り専用 fail-closed を維持し、desktop のみ最小 IPC と実装前提を順に満たして保存 UI を接続する段階設計を記録する。
- [0254-release-operations-disclosure](docs/decisions/0254-release-operations-disclosure.md)
  - 導入・更新・ロールバックと既知問題を先に開示し、公開・署名・endpoint は未決定として残す決定を記録する。
- [0255-epub-test-split](docs/decisions/0255-epub-test-split.md)
  - EPUB の 92 tests を epub/tests.rs へ切り出し、本編のモジュール分割は未決定として残す決定を記録する。
- [0256-wasm-ai-cluster-removal](docs/decisions/0256-wasm-ai-cluster-removal.md)
  - 未使用の fail-closed スタブ 3 型と自己言及 test を削除し、ADR-0156 の AI 部分のみ supersede する決定を記録する。
- [0257-wasm-test-split](docs/decisions/0257-wasm-test-split.md)
  - wasm-bindgen-test 14 件を wasm/tests.rs へ切り出し、本編のモジュール分割は未決定として残す決定を記録する。
- [0258-workspace-feature-additive-deps](docs/decisions/0258-workspace-feature-additive-deps.md)
  - chrono / uuid / wasm dev tokio の feature 差分（member の加算／限定）を記録し、workspace 化せず現状を設計受容とする決定を記録する。
- [0259-wasm-binding-module-split](docs/decisions/0259-wasm-binding-module-split.md)
  - Wasm バインディング本編を epub / archive / lint モジュールへ分割し、公開パスと fail-closed 契約を維持する決定を記録する。
- [0260-epub-module-split](docs/decisions/0260-epub-module-split.md)
  - EPUB 本編を opf / navigation / text / xmlutil モジュールへ分割し、公開 6 型と fail-closed 契約を維持する決定を記録する。
- [0261-os-keychain-deferred](docs/decisions/0261-os-keychain-deferred.md)
  - OS キーチェーン移行を設計受容として記録し、ADR-0098 の未決定へ集約して今回の実装を見送る決定を記録する。
- [0262-e2e-count-documentation-refresh](docs/decisions/0262-e2e-count-documentation-refresh.md)
  - E2E 総数の current 表記をコード出典（255）へ追従させ、履歴的数値は保持する決定を記録する。
- [0263-e2e-count-gate](docs/decisions/0263-e2e-count-gate.md)
  - Checker(expected=N) を唯一の出典として current 文書表記とのズレを fail-closed で検出する静的ゲートを記録する。
- [0264-tauri-ipc-path-acceptance](docs/decisions/0264-tauri-ipc-path-acceptance.md)
  - withGlobalTauri 未設定で desktop も IPC が動かない可能性を記録し、3 経路を比較してコード変更なしの設計受容を選択する。
- [0265-cross-browser-e2e](docs/decisions/0265-cross-browser-e2e.md)
  - 現行 E2E 255 件を Chrome / Edge で実機実行し、他ブラウザの環境ブロッカーを記録する決定。
- [0266-browser-metadata-manifest](docs/decisions/0266-browser-metadata-manifest.md)
  - verify.sh の manifest に E2E のブラウザ実行ファイル・解決パス・バージョン・ポートを記録する決定。
- [0267-bundle-size-baseline](docs/decisions/0267-bundle-size-baseline.md)
  - verify.sh と同一のビルド条件で現行 WASM 配布物のサイズを実測し、今後の比較可能 before とする決定を記録する。
- [0268-csp-consistency-gate](docs/decisions/0268-csp-consistency-gate.md)
  - meta CSP と desktop config CSP の一致・許可オリジンを fail-closed で検査する静的ゲートを記録する。
- [0269-ipc-cas-contract](docs/decisions/0269-ipc-cas-contract.md)
  - update_config_patch に UI が読んだ revision を必須で渡し、衝突を構造化エラーで判別可能にする決定。
- [0270-tauri-global-ipc-path](docs/decisions/0270-tauri-global-ipc-path.md)
  - withGlobalTauri を有効化し、web の `__TAURI__.core.invoke` 参照を公式グローバル経路へ統一する決定。
- [0271-settings-ui-scope](docs/decisions/0271-settings-ui-scope.md)
  - IPC 契約の完了と DTO / a11y / E2E の未実装を切り分け、UI 接続を完全な ConfigPatch DTO 定義後に留める設計受容。
- [0272-settings-save-ui](docs/decisions/0272-settings-save-ui.md)
  - 完全な ConfigView DTO 往復と Tauri mock E2E を用いて、desktop のみ設定保存 UI を接続する決定を記録する。
- [0273-real-webview-verification-deferred](docs/decisions/0273-real-webview-verification-deferred.md)
  - Tauri CLI 未導入で実 WebView 検証を実行不可と記録し、未実施のまま pass とせず導入後の手順を残す決定。
- [0274-a05-traceability-reconciliation](docs/decisions/0274-a05-traceability-reconciliation.md)
  - A05 の前提別状態を再分類し、JS / IPC 契約レベル達成と実 WebView 未達を切り分ける。
- [0275-settings-notice-visibility](docs/decisions/0275-settings-notice-visibility.md)
  - desktop で設定保存 UI が有効なとき、ブラウザ専用の読み取り説明を DOM から除外する決定を記録する。
- [0276-investigation-artifacts-status](docs/decisions/0276-investigation-artifacts-status.md)
- [0277-updater-honesty-restoration](docs/decisions/0277-updater-honesty-restoration.md)
- [0278-settings-ipc-capability-scope](docs/decisions/0278-settings-ipc-capability-scope.md)
- [0279-updater-check-pattern](docs/decisions/0279-updater-check-pattern.md)
- [0280-tauri-plugin-config-cleanup](docs/decisions/0280-tauri-plugin-config-cleanup.md)
- [0281-local-root-flags-server-authoritative](docs/decisions/0281-local-root-flags-server-authoritative.md)
- [0282-real-webview-runtime-proof](docs/decisions/0282-real-webview-runtime-proof.md)
- [0283-webdriverio-adoption](docs/decisions/0283-webdriverio-adoption.md)
- [0284-config-load-fail-closed](docs/decisions/0284-config-load-fail-closed.md)
- [0285-fs-read-only-enforcement](docs/decisions/0285-fs-read-only-enforcement.md)
- [0286-storage-destructive-guard](docs/decisions/0286-storage-destructive-guard.md)
- [0287-ipc-error-redaction](docs/decisions/0287-ipc-error-redaction.md)
- [0288-home-navigation](docs/decisions/0288-home-navigation.md)
- [0289-desktop-frontend-build-dependency](docs/decisions/0289-desktop-frontend-build-dependency.md)
- [0290-ci-reproducibility](docs/decisions/0290-ci-reproducibility.md)
- [0291-e2e-count-single-source](docs/decisions/0291-e2e-count-single-source.md)
- [0292-ai-empty-response](docs/decisions/0292-ai-empty-response.md)
- [0293-lint-retry-feedback](docs/decisions/0293-lint-retry-feedback.md)
- [0294-audit-db-freshness](docs/decisions/0294-audit-db-freshness.md)
- [0295-documentation-current-state](docs/decisions/0295-documentation-current-state.md)
- [0296-wdio-save-roundtrip](docs/decisions/0296-wdio-save-roundtrip.md)
- [0297-front-matter-byte-preservation](docs/decisions/0297-front-matter-byte-preservation.md)
- [0298-atomic-document-save](docs/decisions/0298-atomic-document-save.md)
- [0299-ai-service-refresh-race](docs/decisions/0299-ai-service-refresh-race.md)
  - AI service 生成中に config が更新されても古い service で上書きしない決定。
  - save_document を temp + rename 方式へ変更し、書き込み失敗時に既存ファイルを保持する決定。
  - front matter を構造化値と元テキストの両方で保持し、無編集 open→save でバイト列が変わらないようにする決定。
  - WebdriverIO の settings spec に update_config_patch の revision +1 と config.toml 永続化を追加する決定。
  - 実 WebView 検証済み、EPUB/ZIP 実装済み、E2E current 256 を入口文書へ反映する決定。
  - verify.sh の cargo audit が古い advisory DB で脆弱性を見逃さないよう、DB 鮮度と監査対象の一致を fail-closed で検査する決定。
  - Table / FrontMatter の lint 不合格時に同一 prompt の無駄な AI 再呼び出しを行わない決定。
  - choices / content / response が欠落した応答を空文字成功とせず、固定文言の provider エラーへ変換する決定。
  - E2E 期待件数を Checker(expected=N) のみに依存させ、todo.md の CI-H01 行をゲート条件から外す決定。
  - wasm-pack 固定、lock 不変検証、dependabot npm、WDIO cleanup の安全化で CI の再現性を高める決定。
  - tauri.conf.json の frontendDist 依存を明示化し、生成済み web/dist の存在と安全性を fail-closed で検証する決定。
  - Reader / Editor への導線を追加し、未接続ボタンとダミー FileTree を誤解のない表示へ変更する決定。
  - desktop command の生エラー文字列を廃し、固定 code と固定文言だけを IPC へ返す決定。
  - storage root 自身の削除・改名を拒否し、move_item の上書き競合で既存データを保持する決定。
  - local_roots[].read_only を save_document だけでなく fs コマンドにも適用し、入れ子 root と symlink の迂回を塞ぐ決定。
  - 設定読み込みエラーを握り潰して既定値で上書きする経路を廃し、退避と保存拒否を義務付ける決定。
  - Tauri 公式推奨の WebdriverIO と @wdio/tauri-service を debug 限定で導入し、xdotool に依存しない実 WebView 検証を確立する決定。
  - Tauri CLI 導入後に実 WebView で起動ブロッカーと保存ブロッカーを修正し、設定保存を検証した決定を記録する。
  - 設定 DTO で read_only / watch が writable でも、マージ時に現在値を復元して権限昇格を防ぐ決定を記録する。
  - tauri.conf.json の plugins.*.scope.allowedPaths が無効キーであるため、plugins ブロックを削除し再混入を静的ゲートで防ぐ決定を記録する。
  - 静的ゲートが Tauri v2 の実呼び出し `.check()` を見逃す fail-open を修正する決定を記録する。
  - 設定画面から呼ぶ Tauri command を get_config / update_config_patch に限定する静的ゲートを記録する。
  - 設定保存 UI の editable 化で失われた ADR-0222 の固定表示と、check_updates の保存時 fail-closed を復元する決定。
  - 未追跡の調査レポート群を当時スナップショットとして導線に明示し、生ファイルは変更しない決定を記録する。

## ライセンス

`AGPL-3.0-or-later`（[LICENSE](LICENSE) を参照）。以前の MIT ライセンス本文は
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) に保持しています。
