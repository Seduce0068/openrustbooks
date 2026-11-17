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
| **Markdown処理** | `pulldown-cmark`, `comrak`, `rumdl` (Wasm対応) |
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
- **ドキュメント閲覧 (設計段階)**: EPUB, ZIP, PDF, TXT, Markdown
- **テキスト編集 (設計段階)**: TXT, Markdown (CodeMirror 6ベース)
- **AI支援 (設計段階)**: Markdown生成・修正・リファクタリング +
  `rumdl`品質保証
- **クロスプラットフォーム**: デスクトップ (Tauri) + ブラウザ (Wasm)
- **フォルダ操作 (設計段階)**: 既存フォルダ直接参照, インポート不要
- **クラウド連携 (設計段階)**: Google Drive (手動保存から開始)

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

push（`master`）と pull request では、GitHub Actions がローカルの 6 ゲートと
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

`./scripts/verify.sh` で、上記 6 ゲート・`trunk build`・E2E をまとめて実行できる
（一時出力先は自動で削除される）。

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
- [0001-use-system-webview](docs/decisions/0001-use-system-webview.md) - デスクトップ版にOS WebViewを採用する
- [0002-ai-markdown-pipeline](docs/decisions/0002-ai-markdown-pipeline.md) - AI支援Markdown生成・修正パイプラインを導入する
- [0003-native-wasm-target-split](docs/decisions/0003-native-wasm-target-split.md) - 共通コアをターゲット別依存でNative/Wasm両対応にする
- [0004-zip-archive-reader](docs/decisions/0004-zip-archive-reader.md) - ZIPアーカイブ閲覧をcoreの読み取り専用モジュールとして実装する
- [0005-epub-body-reading](docs/decisions/0005-epub-body-reading.md) - EPUB本文の読み出しをcoreの読み取り専用モジュールとして実装する
- [0006-epub-table-of-contents](docs/decisions/0006-epub-table-of-contents.md) - EPUBの目次（nav / NCX）を平坦な深さ付きリストで返す
- [0007-epub-toc-fragments](docs/decisions/0007-epub-toc-fragments.md) - EPUB目次のフラグメントを保持する
- [0008-epub-cover-and-chapter-title](docs/decisions/0008-epub-cover-and-chapter-title.md) - EPUBの表紙画像と章の表示名を取得する
- [0009-epub-landmarks-and-page-list](docs/decisions/0009-epub-landmarks-and-page-list.md)
  - EPUB3 nav の landmarks と page-list を読む
- [0010-epub-wasm-bindings](docs/decisions/0010-epub-wasm-bindings.md) - EPUB読み出し API を WebAssembly に公開する
- [0011-web-epub-reader](docs/decisions/0011-web-epub-reader.md) - Web 版でローカル EPUB をテキスト閲覧する
- [0012-epub-html-display](docs/decisions/0012-epub-html-display.md) - EPUB の HTML をサニタイズして表示する
- [0013-epub-display-hardening](docs/decisions/0013-epub-display-hardening.md) - EPUB 表示経路を fail-closed に強化する
- [0014-relative-chapter-links](docs/decisions/0014-relative-chapter-links.md) - 本文内の相対リンクから章へ移動する
- [0015-reading-position](docs/decisions/0015-reading-position.md) - 読書位置を localStorage に保存して復元する
- [0016-scroll-position](docs/decisions/0016-scroll-position.md) - 章内のスクロール位置も読書位置として保存する
- [0017-navigation-history](docs/decisions/0017-navigation-history.md) - 章と位置の移動履歴（戻る・進む）
- [0018-reader-display-settings](docs/decisions/0018-reader-display-settings.md) - リーダーの表示設定と読書データの削除
- [0019-position-retention](docs/decisions/0019-position-retention.md) - 読書位置の保持期間と件数上限
- [0020-layout-shift-correction](docs/decisions/0020-layout-shift-correction.md) - 画像読み込みによるレイアウトずれの補正
- [0021-keyboard-and-aria](docs/decisions/0021-keyboard-and-aria.md) - キーボード操作と ARIA 属性
- [0022-position-export-import](docs/decisions/0022-position-export-import.md) - 読書位置のエクスポート・インポート
- [0023-app-wide-theme](docs/decisions/0023-app-wide-theme.md) - アプリ全体のダークテーマ
- [0024-history-persistence](docs/decisions/0024-history-persistence.md) - 章の移動履歴の永続化
- [0025-touch-gestures](docs/decisions/0025-touch-gestures.md) - タッチ操作（スワイプで章を移動）
- [0026-settings-export-import](docs/decisions/0026-settings-export-import.md) - 設定のエクスポート・インポート
- [0027-landmarks-ui](docs/decisions/0027-landmarks-ui.md) - landmarks の移動 UI
- [0028-page-list-navigation](docs/decisions/0028-page-list-navigation.md) - page-list によるページ移動
- [0029-history-list-ui](docs/decisions/0029-history-list-ui.md) - 履歴一覧の UI
- [0030-history-export-import](docs/decisions/0030-history-export-import.md) - 履歴のエクスポート・インポート
- [0031-history-clear](docs/decisions/0031-history-clear.md) - 履歴のクリア
- [0032-import-merge](docs/decisions/0032-import-merge.md) - インポート時のマージ
- [0033-prefers-color-scheme](docs/decisions/0033-prefers-color-scheme.md) - OS のカラースキームへの追従
- [0034-async-image-decoding](docs/decisions/0034-async-image-decoding.md) - 本文画像の非同期デコード
- [0035-history-delete-entry](docs/decisions/0035-history-delete-entry.md) - 履歴エントリの個別削除
- [0036-export-versioning](docs/decisions/0036-export-versioning.md) - エクスポート形式のバージョニング
- [0037-scroll-order-test-stability](docs/decisions/0037-scroll-order-test-stability.md) - スクロール順序テストの安定化
- [0038-initial-theme-follow](docs/decisions/0038-initial-theme-follow.md) - 初回起動時のカラースキーム追従デフォルト
- [0039-forced-colors](docs/decisions/0039-forced-colors.md) - 強制カラーモードへの対応
- [0040-delete-confirmation-dialog](docs/decisions/0040-delete-confirmation-dialog.md) - 削除確認ダイアログ
- [0041-history-summary](docs/decisions/0041-history-summary.md) - 履歴の日時サマリー表示
- [0042-wasm-clippy-cleanup](docs/decisions/0042-wasm-clippy-cleanup.md) - wasm32 clippy 警告の解消
- [0043-svg-blob-sanitization](docs/decisions/0043-svg-blob-sanitization.md) - SVG Blob URL のサニタイズ
- [0044-svg-external-references](docs/decisions/0044-svg-external-references.md) - SVG の外部リソース参照制御
- [0045-svg-xml-processing-instructions](docs/decisions/0045-svg-xml-processing-instructions.md) - SVG の XML 処理命令除去
- [0046-fragment-scroll-stability](docs/decisions/0046-fragment-scroll-stability.md) - フラグメントスクロールの安定化
- [0047-dialog-esc-test](docs/decisions/0047-dialog-esc-test.md) - 確認ダイアログの Esc キャンセル検証
- [0048-dialog-focus-test](docs/decisions/0048-dialog-focus-test.md) - 確認ダイアログのフォーカス検証
- [0049-export-import-roundtrip](docs/decisions/0049-export-import-roundtrip.md) - エクスポート・インポートのラウンドトリップ検証
- [0050-deep-nesting-safety](docs/decisions/0050-deep-nesting-safety.md) - 深い入れ子 XHTML の安全性確認
- [0051-owned-epub-open](docs/decisions/0051-owned-epub-open.md) - EPUB バイト列の所有権移譲による二重コピー解消
- [0052-fragment-retry-hardening](docs/decisions/0052-fragment-retry-hardening.md) - フラグメントスクロール retry の強化と残る課題
- [0053-scroll-reset-root-cause](docs/decisions/0053-scroll-reset-root-cause.md) - フラグメントスクロールが 0 に戻る根本原因
- [0054-early-generation-check](docs/decisions/0054-early-generation-check.md) - 画像待ちループの早期世代チェック
- [0055-same-chapter-fragment-history](docs/decisions/0055-same-chapter-fragment-history.md) - 同一章フラグメント移動の履歴記録
- [0056-swipe-threshold-setting](docs/decisions/0056-swipe-threshold-setting.md) - スワイプ閾値の設定化
- [0057-content-language](docs/decisions/0057-content-language.md) - 本文言語の設定
- [0058-region-aria-labelledby](docs/decisions/0058-region-aria-labelledby.md) - 本文領域のアクセシブル名
- [0059-malformed-xhtml-policy](docs/decisions/0059-malformed-xhtml-policy.md) - 非整形式 XHTML の扱い
- [0060-remote-resources-policy](docs/decisions/0060-remote-resources-policy.md) - remote-resources 宣言の扱い
- [0061-vertical-scroll-threshold](docs/decisions/0061-vertical-scroll-threshold.md) - 縦スクロール閾値の設定化
- [0062-browser-pinch-zoom](docs/decisions/0062-browser-pinch-zoom.md) - ブラウザ標準のピンチズーム
- [0063-legacy-roundtrip](docs/decisions/0063-legacy-roundtrip.md) - 旧形式とマージを含むラウンドトリップ検証
- [0064-large-image-warning](docs/decisions/0064-large-image-warning.md) - 巨大画像の検出と警告
- [0065-deep-nesting-10k](docs/decisions/0065-deep-nesting-10k.md) - 10000 階層の入れ子テスト
- [0066-remote-resources-warning](docs/decisions/0066-remote-resources-warning.md) - remote-resources 宣言の警告
- [0067-optional-image-downscale](docs/decisions/0067-optional-image-downscale.md) - オプトインの画像縮小
- [0068-reset-settings](docs/decisions/0068-reset-settings.md) - 表示設定の一括リセット
- [0069-timestamped-export-filename](docs/decisions/0069-timestamped-export-filename.md) - エクスポート名のタイムスタンプ
- [0070-export-filename-timezone](docs/decisions/0070-export-filename-timezone.md) - エクスポート名へのタイムゾーン明示
- [0071-downscale-mime-preservation](docs/decisions/0071-downscale-mime-preservation.md) - 縮小画像の MIME 保持
- [0072-downscale-jpeg-quality](docs/decisions/0072-downscale-jpeg-quality.md) - 縮小 JPEG の品質指定
- [0073-jpeg-quality-setting](docs/decisions/0073-jpeg-quality-setting.md) - JPEG 品質の設定化
- [0074-jpeg-size-comparison](docs/decisions/0074-jpeg-size-comparison.md) - JPEG 品質とファイルサイズの定量比較の調査
- [0075-shortcuts-help](docs/decisions/0075-shortcuts-help.md) - キーボードショートカットのヘルプ
- [0076-version-migration-policy](docs/decisions/0076-version-migration-policy.md) - エクスポート形式のバージョン移行方針
- [0077-version-1-sufficiency](docs/decisions/0077-version-1-sufficiency.md) - エクスポート形式 version 1 の十分性
- [0078-chapter-filter](docs/decisions/0078-chapter-filter.md) - 章タイトルの検索フィルター
- [0079-filter-focus-shortcut](docs/decisions/0079-filter-focus-shortcut.md) - 章フィルターへのフォーカスショートカット
- [0080-external-link-safety-test](docs/decisions/0080-external-link-safety-test.md) - 外部リンク安全性の E2E 検証
- [0081-external-link-popup-test](docs/decisions/0081-external-link-popup-test.md) - 外部リンクのポップアップ呼び出し検証
- [0082-mailto-link-test](docs/decisions/0082-mailto-link-test.md) - mailto リンクの検証
- [0083-dead-code-cleanup](docs/decisions/0083-dead-code-cleanup.md) - デッドコードと古い契約値の整理
- [0084-verification-script](docs/decisions/0084-verification-script.md) - 一括検証スクリプト
- [0085-single-entry-history-save](docs/decisions/0085-single-entry-history-save.md) - 履歴が 1 件になっても localStorage を更新する決定を記録する。
- [0086-dependency-vulnerability-fix](docs/decisions/0086-dependency-vulnerability-fix.md) - ammonia と pulldown-cmark の脆弱性を修正する決定を記録する。
- [0087-front-matter-preservation](docs/decisions/0087-front-matter-preservation.md) - Markdown 文書の保存時に front matter が失われるバグを修正する決定を記録する。
- [0088-wasm-path-stripping](docs/decisions/0088-wasm-path-stripping.md) - 配布 WASM に含まれる開発ホスト絶対パスを除去する決定を記録する。
- [0089-init-unification](docs/decisions/0089-init-unification.md) - Rust / JS 双方の init() が同じ初期化処理を呼ぶようにする決定を記録する。
- [0090-e2e-expected-count](docs/decisions/0090-e2e-expected-count.md) - 未実行の check を見逃さないため、期待総数を宣言して検証する決定を記録する。
- [0091-content-security-policy](docs/decisions/0091-content-security-policy.md) - Web 版に CSP meta を追加し、リソース読み込み先を制限する決定を記録する。
- [0092-ime-composing-guard](docs/decisions/0092-ime-composing-guard.md) - IME の変換中はキーボードショートカットを発火させない決定を記録する。
- [0093-modal-shortcut-guard](docs/decisions/0093-modal-shortcut-guard.md) - dialog が開いているときは背景のキーボードショートカットを止める決定を記録する。
- [0094-desktop-window-result](docs/decisions/0094-desktop-window-result.md)
  - get_webview_window の unwrap() を Result 化し、起動時の panic を防ぐ決定を記録する。
- [0095-empty-export-and-text-shrink](docs/decisions/0095-empty-export-and-text-shrink.md)
  - 保存データ 0 件のエクスポート経路と文字縮小操作を E2E で検証し、空ファイル生成を防ぐ決定を記録する。
- [0096-appstate-tests](docs/decisions/0096-appstate-tests.md)
  - desktop の AppState に Tauri ランタイム不要の状態遷移テストを追加する決定を記録する。
- [0097-path-containment](docs/decisions/0097-path-containment.md)
  - LocalStorage::resolve_path を fail-closed 化し、root 外へのパス脱出を拒否する決定を記録する。
- [0098-secret-exposure-mitigation](docs/decisions/0098-secret-exposure-mitigation.md)
- [0099-ai-default-off](docs/decisions/0099-ai-default-off.md)
- [0100-ai-service-guard](docs/decisions/0100-ai-service-guard.md)
- [0101-base-url-allowlist](docs/decisions/0101-base-url-allowlist.md)
- [0102-extra-headers-redaction](docs/decisions/0102-extra-headers-redaction.md)
- [0103-path-field-redaction](docs/decisions/0103-path-field-redaction.md)
- [0104-send-consent-dialog](docs/decisions/0104-send-consent-dialog.md)
- [0105-retry-config](docs/decisions/0105-retry-config.md)
- [0106-retry-merge-preservation](docs/decisions/0106-retry-merge-preservation.md)
- [0107-ai-service-refresh](docs/decisions/0107-ai-service-refresh.md)
- [0108-prompt-system-prompt](docs/decisions/0108-prompt-system-prompt.md)
- [0109-system-prompt-unification](docs/decisions/0109-system-prompt-unification.md)
- [0110-lint-auto-fix](docs/decisions/0110-lint-auto-fix.md)
- [0111-fix-convergence](docs/decisions/0111-fix-convergence.md)
- [0112-lazy-image-loading](docs/decisions/0112-lazy-image-loading.md)
- [0113-image-settle-hardening](docs/decisions/0113-image-settle-hardening.md)
- [0114-expected-image-count](docs/decisions/0114-expected-image-count.md)
- [0115-epub-css-policy](docs/decisions/0115-epub-css-policy.md)
- [0116-final-lint-policy](docs/decisions/0116-final-lint-policy.md)
- [0117-wasm-config-alignment](docs/decisions/0117-wasm-config-alignment.md)
- [0118-pipeline-dead-code](docs/decisions/0118-pipeline-dead-code.md)
- [0119-wasm-test-runner](docs/decisions/0119-wasm-test-runner.md)
- [0120-retry-limit](docs/decisions/0120-retry-limit.md)
- [0121-req010-retry-wording](docs/decisions/0121-req010-retry-wording.md)
  - REQ-010 の「最大3回再生成」を実装と一致する文言に明文化する決定を記録する。
  - max_retries を service 境界で 0..=10 にクランプし、暴走とオーバーフローを防ぐ決定を記録する。
  - wasm-bindgen-test と wasm-pack により wasm32 単体テストを実際に実行するゲートを追加する決定を記録する。
  - execute_pipeline の到達不能な last_error と動的 Err を削除する決定を記録する。
  - WasmConfig の ai_enabled 既定値を false にし、native の AiConfig と揃える決定を記録する。
  - AI pipeline の最終試行で lint 不合格でも Ok を維持し、呼び出し側が lint_passed を確認して差分を提示する方針を記録する。
  - EPUB 内 CSS は現状どおり適用しないことを維持し、将来のオプトイン・スコープ付き段階導入の方針を記録する。
  - expected_images を src を持つ画像の数にし、src 削除画像による無駄な 2 秒待ちを解消する決定を記録する。
  - images_settled() に自然幅の確認を追加し、complete() だけでは見逃す画像を待つ決定を記録する。
  - IntersectionObserver による画像遅延読み込みは全面切替を見送り、寸法予約→段階導入の順で検討する決定を記録する。
  - rumdl fix の未収束情報を保持し、未収束時は修正を採用しない決定を記録する。
  - rumdl の fix API を用いて Markdown lint の自動修正を実装し、auto_fix 設定を配線する決定を記録する。
  - OpenAI の固定 system message を削除し、system 指示をプロンプトテンプレートに一元化する決定を記録する。
  - 内蔵 .hbs テンプレートに必須フィールドを追加し、AI サービスの初期化失敗を解消する決定を記録する。
  - AI/lint 設定変更時に service を fail-closed で再初期化する決定を記録する。
  - max_retries を DTO から除外し、IPC 往復で現在値を保持する決定を記録する。
  - リトライ回数をトークン上限から分離し、AiConfig.max_retries に集約する決定を記録する。
  - 本文を AI へ送る直前に Rust 側でネイティブ確認を要求する決定を記録する。
  - 設定 IPC の DTO からローカルパス情報を除外し、マージで保持する決定を記録する。
  - OpenAiConfig.extra_headers を IPC DTO と Debug 出力からマスクする決定を記録する。
  - AI の送信先 URL を既定ホストと loopback のみに制限し、JS からの任意送信先差し替えを防ぐ決定を記録する。
  - AI を無効化した後に service が残存して送信できる問題を修正する決定を記録する。
  - AI 機能を既定で無効にし、明示的な opt-in まで本文を送信しないようにする決定を記録する。
  - API キー等の IPC 露出をマスクし、Debug とファイル権限を硬化する決定を記録する。

## ライセンス

`AGPL-3.0-or-later`（[LICENSE](LICENSE) を参照）。以前の MIT ライセンス本文は
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) に保持しています。
