---
type: Proposal
title: "openrustbooksの構成案"
description: "Rustを中心とする構成候補、環境別の境界、開発順序の提案。"
tags:
  - architecture
  - Rust
  - Tauri
status: proposed
---

> **文書の位置づけ（2026-11-17 時点）:** 本文は初期提案時点の構成案で、
> 現在の実装状況とは一部異なり、status も `proposed` のままである。
> 採用済みの技術・方針は [README](../README.md) と
> `docs/decisions/` の ADR を参照（例: Leptos 0.8、Tauri 2、rumdl を採用済み）。
> 本ファイルは提案時の判断記録として残す。

## 決定と候補

| 項目 | 状態 | 内容 |
| --- | --- | --- |
| Rustの活用 | 利用者方針 | 文書処理などで積極的に利用する |
| デスクトップ基盤 | 決定 | TauriのOS WebView方式（wry） |
| Tauriのバージョン | 未決定 | 実装開始時に選定する |
| Web画面 | 候補 | Leptos。Yewも比較候補 |
| 共通処理 | 提案 | OS依存を分離したRustの共通コア |
| サーバー | 候補 | 必要になった場合にaxumを検討 |
| PDF描画 | 候補 | PDF.js |
| テキスト編集部品 | 候補 | CodeMirror |
| Markdown lint | 候補 | markdownlintエンジン |
| 高機能プレビュー | 候補 | Crossnote再利用、機能単位の実装 |

Tauri・Leptos/Yew・axumは異なる層を担当する。
一つだけを選ぶ関係ではない。

[Tauriのフロントエンド構成][tauri-frontend]では、
HTML・CSS・JavaScript・Wasmで構成する画面を利用できる。
[Leptos][leptos]や[Yew][yew]で画面をRustで書くことは可能だが、
JavaScript部品との連携も選択肢に残す。

## 共通化の境界

| 層 | 担当の案 | 環境依存の扱い |
| --- | --- | --- |
| 画面 | ファイルツリー、閲覧、編集、状態表示 | 共通化を目指す |
| Rust共通コア | 文書モデル、アーカイブ、検索処理 | OS APIへの直接依存を避ける |
| 入出力 | データ取得、保存、一覧、変更検知 | 保存先・環境ごとに実装 |
| ネイティブ連携 | OSのフォルダ表示など | Tauri側で担当 |
| 任意のサーバー | 本棚APIや同期API | axumの採用を別途判断 |

ZIPのバイト列を解析する処理と、そのZIPを取得する処理を分離する。
各crateのWasm対応、必要な機能フラグ、メモリ使用量は採用前に検証する。
Rustのコードなら自動的にすべてWasmで動くとは扱わない。

デスクトップ版ではネイティブRust、
ブラウザ版ではWasmで共通処理を利用する案。
処理を画面スレッドから分離する必要性は実測して判断する。

## ブラウザとデスクトップ

Tauri自体がブラウザ版を提供するわけではない。
共通画面をWebとして配信する経路を別に用意する。

ブラウザの[showDirectoryPicker()][directory-picker]には対応差がある。
フォルダの直接読み書きは、対象ブラウザと権限の確認が必要。
未対応環境向けにはファイル選択、ZIP入出力、Drive経由を候補にする。

ブラウザの内部保存領域は、OSから普通に開ける文書フォルダと同一ではない。
ブラウザ版だけでOSのファイルマネージャー操作を同等に提供すると約束しない。

デスクトップ版では[Tauri Opener][opener]をOS連携の候補にする。
CEFやElectronによるChromium同梱は採用方針に含めない。
根拠は[ADR-0001](decisions/0001-use-system-webview.md)を参照。

## 保存先とフォルダ

通常の文書・画像を通常のファイルとして保存し、
他のアプリでも利用できる構成を提案する。
検索用データや読書位置は、本文と分けて管理する案がある。

保存先の実装は、一覧・読み込み・書き込みだけでなく、
書き込み可否、変更検知、競合検知、OSで開けるかを表現できるようにする。
クラウドとローカルで同じ操作が常に可能とは扱わない。

Google Driveでは[通常フォルダ][drive-folder]を本文の保存先候補とする。
[appDataFolder][drive-appdata]はユーザーから隠れるため、
エクスプローラーから文書を開く要件の保存先としては適さない。

Drive APIだけでクラウドフォルダのローカルパスができるわけではない。
OSのファイルマネージャーで開くには、
同期済みの場所やローカルへの保存を別途設計する。

認可範囲は[Drive APIのスコープ][drive-scopes]を確認して決める。
フォルダ選択だけで配下のすべての既存ファイルを扱えると仮定せず、
必要な操作と権限を試作で確認する。

## 形式別の処理

[EPUB 3.3仕様][epub]を参照し、
アーカイブ構造、パッケージ情報、読み順、本文のHTML/CSSを扱う。
本文の描画にはWebの描画機能を使う案。
対応するEPUBの範囲と、既存ライブラリの採用は未決定。

PDFには[PDF.js][pdfjs]を候補とする。
PDFの編集・OCRなどを初期閲覧機能と混同しない。

ZIPは内部のファイルを仮想的なツリーとして表示する案。
編集時は、展開先への保存とアーカイブ再作成を区別する。
アーカイブの更新中に失敗した場合の扱いも仕様化する。

## Markdown編集とプレビュー

[CodeMirror][codemirror]は編集部品の候補。
日本語IME、選択、Undo、モバイルでの操作を試作で確認する。

[markdownlint][markdownlint]にはブラウザ向けの利用方法がある。
既存エンジンを用いて診断を表示する案とし、
設定ファイル・ルール・修正機能の対応範囲は別途決める。

[MPEの基盤であるCrossnote][crossnote]は再利用の候補。
数式、Mermaid、プレビュー同期、外部コマンド実行、各種エクスポートを
一括して対応済みとは扱わず、必要な機能と実行環境を整理する。

[Markdown All in One][all-in-one]はVS Code拡張。
ショートカット、目次生成、リスト継続などの必要な機能を具体化する。
VS Code拡張ホストを丸ごと実装する方針は決定していない。

## axumが必要になる場合

[axum][axum]はHTTPサーバー・APIを作るための候補。
ブラウザ版を作るだけで必須になるものではない。
[Leptosの公式サーバー連携][leptos-ssr]にはaxumが含まれる。

候補となる用途は以下。

- PCやNASの本棚を、別端末のブラウザから利用する。
- アプリ独自の同期APIやアカウント機能を提供する。
- ブラウザで完結しない処理をサーバー側で提供する。

axumがアクセスするファイルは、サーバーの実行環境にあるもの。
クラウド上のサーバーから利用者のローカルフォルダを直接開けるとは扱わない。

## 開発順序の提案

各段階の順序と範囲は未確定。
先に操作性と環境差を確かめ、後続の要件を削除せず段階的に実装する。

1. フォルダを開き、TXT・Markdownを閲覧・編集・保存する試作。
2. 共通画面をデスクトップとブラウザで動かし、日本語入力を確認。
3. Markdownの相対リンク・画像・lint・必要なプレビュー機能を検証。
4. ZIP・EPUB・PDFの閲覧を実装し、対応サンプルで検証。
5. Google Driveの読み書き、フォルダ保存、アーカイブ保存を検証。
6. 必要に応じて自動同期、オフライン、高度なプレビューを追加。

OS WebViewの差に影響する表示・入力は、対象環境で確認する。
数値による性能要件や対応ファイルの上限は、まだ決まっていない。

## AI Markdown パイプライン

[ADR-0002](../decisions/0002-ai-markdown-pipeline.md) で決定。

### 共通コアへの組み込み

| モジュール | 責務 | Wasm対応 |
| --- | --- | --- |
| `ai_markdown::pipeline` | 生成・修正・検証の往復制御 | 必須 |
| `ai_markdown::prompt` | プロンプトテンプレート管理・バージョニング | 必須 |
| `ai_markdown::client` | AIプロバイダ抽象化 (`AiClient` trait) | 必須 |
| `ai_markdown::lint` | `rumdl` ラッパー・ルールセット適用・自動修正 | 必須 |
| `ai_markdown::diff` | 差分計算・インライン表示用データ生成 | 必須 |

### 依存クレート

- `async-openai` — AI API クライアント
- `tiktoken-rs` — トークン計算
- `similar` — 差分アルゴリズム
- `comrak` — Markdown→HTML 変換 (`default-features = false`)、
  `ammonia` — 生成 HTML のサニタイズ（二段構え）、`rumdl` — markdownlint 互換 lint

### プロンプト戦略

- **ファイルベース管理**: `prompts/{generate,refine,table,frontmatter}.hbs` (Handlebars風)
- **バージョニング**: Git 管理・セマンティックバージョン付与
- **注入変数**: 対象文書・ユーザー指示・プロジェクト設定 (`.markdownlint.yaml`)・文脈 (カーソル位置等)

### UI 統合案 (CodeMirror 6 拡張)

- コマンドパレット: `AI: Generate Markdown`, `AI: Refine Selection`
- インラインアクション: 選択範囲右クリック → `AI 改善`
- ストリーミング表示: 生成途中もトークン単位でエディタ反映
- 差分ビュー: `original` / `proposed` 並列表示・チェリーピック採用

### リスク検証項目 (優先順)

1. `rumdl` Wasm ビルドサイズ・初期化速度・メモリ使用量
2. AI API レイテンシ・コスト・ストリーミング対応 (ローカルLLM含む)
3. プロンプトインジェクション耐性・出力検証の堅牢性
4. 大容量文書 (100KB+) へのスケーラビリティ

## 文書の運用

要件は[requirements.md](requirements.md)、
合意した技術判断は[decisions](decisions/)に記録する。
提案を採用するときは状態と判断理由を更新する。

将来、利用手順を追加するときは目的・前提・操作・検証・
トラブルシュートを整理する。
文書が増えた段階で[Diátaxis][diataxis]の区分を用い、
表記は[Googleの文書スタイルガイド][google-style]を参考にする提案。

[tauri-frontend]: https://v2.tauri.app/start/frontend/
[leptos]: https://leptos.dev/
[yew]: https://yew.rs/
[directory-picker]: https://developer.mozilla.org/en-US/docs/Web/API/Window/showDirectoryPicker
[opener]: https://v2.tauri.app/plugin/opener/
[drive-folder]: https://developers.google.com/workspace/drive/api/guides/folder
[drive-appdata]: https://developers.google.com/workspace/drive/api/guides/appdata
[drive-scopes]: https://developers.google.com/workspace/drive/api/guides/api-specific-auth
[epub]: https://www.w3.org/TR/epub-33/
[pdfjs]: https://mozilla.github.io/pdf.js/
[codemirror]: https://codemirror.net/
[markdownlint]: https://github.com/DavidAnson/markdownlint
[crossnote]: https://github.com/shd101wyy/crossnote
[all-in-one]: https://github.com/yzhang-gh/vscode-markdown
[axum]: https://docs.rs/axum/latest/axum/
[leptos-ssr]: https://book.leptos.dev/ssr/index.html
[diataxis]: https://diataxis.fr/
[google-style]: https://developers.google.com/style
