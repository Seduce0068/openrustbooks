---
type: Decision
title: "ADR-0127: エディタ UI と Tauri command の段階接続"
description: "CodeMirror を導入せず、既存の Tauri command を段階的に Web UI へ接続する方針を記録する。"
tags:
  - ADR
  - web
  - desktop
  - ui
generated:
  by: dsh
  at: "2026-11-18T09:10:00+09:00"
status: accepted
---

## 背景

`web/src/lib.rs` の Editor / SettingsForm / Home の各ボタンは未配線で、
`Effect::new` はマウント時にログを出すだけ、Save は `saving` を
true→false にするだけのスタブ。Web 側に Tauri `invoke` は 1 箇所も無い。
一方 desktop 側には 25 の command が登録済みで、frontendDist は
`../web/dist`（同一の Wasm 成果物）を参照する。

## 現状（editor-ui-researcher の調査）

- 未配線: Home の New File / Open Folder / Open File、FileTree、
  Editor の Save / AI Generate / AI Refine / Lint / format select、
  editor-pane（空 div）、SettingsForm の 11 要素と submit、
  Preview の `content` 更新、desktop の `document-*` イベント受信。
- `desktop/src/main.rs:98-129` に 25 command 登録済み
  （document 6 / fs 5 / ai 6 / config 5 / window 3）。
- 外部検索は全手段が失敗（HTTP 402 / timeout / fetch failed）ため、
  外部ライブラリのVersion・サイズ・ライセンスは **未確認／推測**。

## 決定

- **CodeMirror は今回導入しない**。まず `<textarea>` ＋ Leptos シグナルで
  入力・保存・Undo（ブラウザ標準）を実現する最小構成を検討する。
  `web-sys` の `HtmlTextAreaElement` feature は既に有効で、
  新規依存は不要（editor-deps-researcher の確認）。
- Web → Rust の呼び出しは、npm 依存を増やさず
  `window.__TAURI*.core.invoke`（global Tauri）を Rust/Wasm 側の薄い
  ラッパから呼ぶ方式を第一候補とする。実現には
  `desktop/tauri.conf.json` の `app.withGlobalTauri: true` が必要
  （外部情報のため **未確認**、実機検証が必要）。
- ブラウザ単体（Wasm）では `window.__TAURI*` が無いことを検出し、
  対象機能を非活性化＋「デスクトップ版のみ」と表示する。
  Wasm の AI / lint は固定値スタブなので接続しない。
- 段階導入の順序:
  1. `get_config` / `update_config`（SettingsForm のバインドと submit）。
     secret は `has_*` フラグと `merge_secrets` で保護済み。
  2. `get_document_content` / `set_document_content` / `save_document` /
     `open_document`（Editor の Save と Home の Open File）。
  3. `lint_markdown` / `lint_and_fix_markdown`（Lint ボタン）。
  4. `generate_markdown` / `refine_markdown`（送信前同意 ADR-0104 と
     結果後の `lint_passed` / diff 確認＋保存／再生成／破棄 ADR-0116 の UI が必須）。
- 見送り: `convert_to_table` / `generate_front_matter` / window 系
  （専用 UI が無い）、fs 系（capability 追加が必要／**推測**）、ZIP
  （Web バインディングが無い）。
- コード変更は今回行わない（調査と決定のみ）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| textarea ＋段階 IPC 接続（採用） | 新規依存なしで最小機能から始められる |
| CodeMirror Rust クレート | 活発性・ライセンス・Wasm サイズが未確認で第一候補にしない |
| wasm-bindgen で JS API 直接 | 将来ハイライトが必要になった時点で検討 |
| 全機能を一括実装 | 同意／結果確認／capability 等が未設計でリスク大 |

## 影響

- 今回はコード変更なし。既存の 216 テスト＋wasm 3、E2E 228 項目は維持。
- 将来の実装では、既存の IME／入力欄フォーカス保护
  （`web/src/lib.rs:2927-2947`）を壊さないこと。
- `desktop/tauri.conf.json` の CSP `script-src` に
  `'wasm-unsafe-eval'` を追加する必要がある可能性（**推測**、
  web/index.html には既に設定済み）。実機で確認する。

## 検証

- 2 観点のエージェント調査（UI 接続状況と依存選定）を実施。
- コード変更が無いためゲートは既存の緑を維持。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未確認／未決定（明示）

- 外部ライブラリ（CodeMirror 本体／Rust ラッパー）の活発性・ライセンス・
  バンドルサイズは、検索失敗のためすべて未確認。
- `withGlobalTauri` の有効化と `__TAURI*` の実際の公開経路、
  `script-src 'wasm-unsafe-eval'` の要否は実機未検証。
- capability 不在時の plugin command の拒否挙動は未確認（**推測**では拒否）。
- 編集中の構文ハイライト／リッチ Markdown 編集が本当に要件か。
- 保存先（localStorage / ファイル / Tauri command）と保存失敗時の扱い。
- Leptos が textarea の value を再描画すると Undo 履歴が消える可能性
  （**推測**、要検証）。
