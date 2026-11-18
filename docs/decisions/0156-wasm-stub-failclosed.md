---
type: Decision
title: "ADR-0156: Wasm AI/lint スタブの fail-closed 化"
description: "ブラウザで AI/lint が成功を偽装していた Wasm API を明示的なエラーに変更する決定を記録する。"
tags:
  - ADR
  - wasm
  - security
generated:
  by: dsh
  at: "2026-11-18T20:30:00+09:00"
status: accepted
---

## 背景

ADR-0151 の探索で、`core/src/wasm.rs` の `WasmAiMarkdownService` が
固定文字列と `lint_passed: true`、架空の `tokens_used` を返し、
`WasmMarkdownLinter` が常に空 issues / `passed: true` を返すことが判明した。
Web UI からは未参照だが、将来接続された際に誤った成功を返す危険がある。

## 決定

- AI メソッド 4 つ（`generate` / `refine` / `to_table` / `front_matter`）は
  `Err(JsValue::from_str("AI features are not available in the browser."))` を返す。
- `WasmMarkdownLinter::lint()` / `lint_and_fix()` は
  `Err(JsValue::from_str("Linting is not available in the browser."))` を返す。
- 架空の `tokens_used` / `lint_passed: true` / 空 issues は返さない。
- エラー文言とログにユーザー入力、プロンプト、キー、値を出さない
  （旧コードの全文 console 出力を文字数のみへ変更）。
- `WasmPipelineResult` / `WasmLintIssue` / `WasmLintFixResult` は
  JS 側の型互換のため残置する（現在は生成されない）。
- real lint（featureless rumdl の直接配線）は今回見送る。
  `JsValue` → `LintConfig` の fail-closed 変換、
  `LintIssue` → `WasmLintIssue` の変換（end_line 等）、
  未収束情報（converged / conflicting_rules）の設計が必要で、
  「明らかに小さく安全」とは言えないため。

## 影響

- 公開 wasm API の契約変更:
  - 同期 `lint()` は JS で**同期 throw**。
  - 非同期 5 メソッド（AI 4 つ + `lint_and_fix()`）は **Promise rejection**。
  - 値は JS 文字列（`JsValue::from_str`）であり `Error` オブジェクトではない。
    将来 UI が `e.message` を読むと undefined になる（**未決定**）。
- 現在の Web UI はこれらを未参照のため回帰なし。
- `WasmConfig` の `lint_enabled: true` / `auto_fix: true` 既定は維持。
  lint API が常に失敗するとの設定契約の矛盾は、フラグを読むコードが
  無いため顕在化していない（**推測**）。
- ADR-0155 の「wasm start 衝突」は、より正確には
  rumdl の `wasm` feature が公開する同名 export `init` との衝突。
  複数 `#[wasm_bindgen(start)]` 自体は 0.2.129 で連結実行される
  （出典: wasm-bindgen 0.2.129 start 仕様）。

## 検証（Runtime Proof）

- `cargo fmt --all -- --check`、
  `cargo test --all-targets --locked`、
  `cargo clippy --all-targets --locked -- -D warnings`、
  wasm32 core check、wasm-pack test --node: すべて成功。
- wasm-bindgen-test は 6 件（新規 2 件:
  `ai_methods_fail_closed` / `lint_methods_fail_closed`、
  既存 4 件は変更なし）。
- `./scripts/verify.sh` rc=0、E2E **233 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- JS 文字列エラーを `js_sys::Error` へ変更し `e.message` を提供するか
  （テスト契約 `as_string()` の変更を伴う）。
- エラー文言の i18n（固定英語のまま）。
- ~~実 lint の設計~~ ADR-0160 で featureless rumdl 接続を実装（未収束情報とサイズ計測は未決定）。
- `lint_enabled` / `auto_fix` 既定値の扱い、能力フラグの追加。
- JS 側で catch / reject ハンドリングを接続する際のテスト追加
  （同期 throw と非同期 reject の取り違え注意）。
