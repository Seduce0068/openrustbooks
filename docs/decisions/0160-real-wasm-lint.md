---
type: Decision
title: "ADR-0160: Wasm Markdown linter の実配線"
description: "fail-closed スタブだった WasmMarkdownLinter を featureless rumdl の実 lint に接続する決定を記録する。"
tags:
  - ADR
  - wasm
  - ai
generated:
  by: dsh
  at: "2026-11-18T21:50:00+09:00"
status: accepted
---

## 背景

ADR-0156 で Wasm AI/lint を fail-closed エラーに変更した。
lint は featureless rumdl が wasm ビルド可能で、
native の `MarkdownLinter` も wasm でコンパイルされるため、
ブラウザでも実 lint を動かせる（ADR-0110 / 0123 / 0156 の未決定）。

## 決定

- `WasmMarkdownLinter` を `MarkdownLinter`（`core/src/ai_markdown/lint.rs`）へ接続:
  - `lint(content)`: `LintIssue` → `WasmLintIssue` 変換して JS 配列を返す。
    `rule` / `rule_name` / `line` / `column` / `end_line` / `end_column` /
    `message` / `severity` / `fixable` を保持。
  - `lint_and_fix(content)`: `passed` / `issues` / `fixed_content` /
    `fixes_applied` を返す。
- config 解釈:
  - `null` / `undefined` は `LintConfig::default()`。
  - それ以外は完全な `LintConfig` のパース成功時のみ使用。
  - 失敗時は `"Invalid lint config: ..."` で **fail-closed**
    （部分オブジェクト `{auto_fix:false}` が既定 `true` へ反転する
    fail-open を防ぐ）。未知キーは serde が無視する。
- `auto_fix` は `LintConfig.auto_fix` が権威。
  native の `validate_content()` と同じ分岐を wasm 側で再現
  （`auto_fix=false` なら lint のみ、本文不変、`fixes_applied=0`）。
- 収束情報（`converged` / `conflicting_rules`）は
  `WasmLintFixResult` に追加せず **omit**。
  未収束は native の `passed=false` に反映されるが、
  JS から「振動」と「反復上限到達」を判別できない（未決定）。
- `issues()` getter の `unwrap()` を `map_err` に置換し、
  本番コードから `unwrap()` を除去。

## 影響

- `WasmMarkdownLinter::new()` が `Result<Self, JsValue>` に変更。
  不正 config は constructor の同期 throw。
- JS 側の呼び出し経路は現状なし（web/ は未参照）のため回帰なし。
- rumdl `wasm` feature は有効化しない（同名 export `init` 衝突回避、
  ADR-0155 / 0156）。featureless を直接利用。
- `WasmConfig.lint_enabled` / `WasmConfig.auto_fix` は依然未参照。
  constructor の `LintConfig` が権威（設定 UI 接続時に要注意）。
- バンドルサイズの before/after は **未計測**（推測を含む）。

## 検証（Runtime Proof）

- wasm-bindgen-test は 10 件（新規 5 件:
  clean / MD033 検出 / lint_and_fix / auto_fix=false / 部分 config 拒否、
  既存 5 件は維持）。
- `cargo test --all-targets --locked`:
  core 196 + desktop 56 + web 2。
- `cargo clippy --all-targets --locked -- -D warnings`、
  wasm32 core check、`./scripts/verify.sh`: すべて rc=0。
- E2E **235 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 追記（2026-11-28）

- JS 側の呼び出し経路は ADR-0357 で追加済み。本 ADR の「現状なし」は当時点の記録。

## 未決定

- `converged` / `conflicting_rules` を JS へ公開するか（additive getter）。
- エラー文言の i18n、`js_sys::Error` 化（`e.message` 提供）。
- 未知キーの拒否方針、部分 config のフィールド単位マージ。
- `WasmConfig.lint_enabled` / `auto_fix` と `LintConfig` の権威関係。
- バンドルサイズ計測とサイズゲート（ADR-0155 から継続）。
  ~~workspace rumdl 残存~~ ADR-0168 で削除済み。
- `conflict_cycle`、反復上限 10 の根拠、`rule_config` 上限、
  severity / enabled のルール単位 support（既存 ADR から継続）。
