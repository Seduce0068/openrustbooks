---
type: Decision
title: "ADR-0194: lint 検証分岐の一元化"
description: "native と wasm で重複していた auto_fix 分岐を lint.rs の共通関数へ集約する決定を記録する。"
tags:
  - ADR
  - core
  - wasm
generated:
  by: dsh
  at: "2026-11-19T07:38:00+09:00"
status: accepted
---

## 背景

`core/src/ai_markdown/mod.rs::validate_content()`（native）と
`core/src/wasm.rs::WasmMarkdownLinter::lint_and_fix()` が
「auto_fix=true なら `lint_and_fix()`、false なら `lint()` +
`LintResultWrapper` 構築」という同じ分岐を各自実装していた（R6 PR-M09）。
片方だけ修正されると native と browser で lint 判定が食い違う。

## 決定

- `lint.rs` に次を追加する:
  `pub async fn validate_with_auto_fix(linter: &MarkdownLinter, content: &str, auto_fix: bool) -> CoreResult<LintResultWrapper>`
- native `validate_content()` は薄いラッパーへ変更する。
- wasm `lint_and_fix()` は共通関数へ委譲し、重複コードを削除する。
  `WasmLintFixResult` への変換と `map_err(js_error)` は境界側に残す。
- 共通関数は `CoreResult` を返し、`JsValue` 化しない（エラー変換は境界の責務）。
- `LintResultWrapper` の形・意味は変更しない。

## 共通関数の契約

- `auto_fix=true`:
  `linter.lint_and_fix().await` の結果をそのまま返す。
  未収束時は `apply_fix_result()` の fail-closed 契約
  （`passed=false`、本文差し戻し、`fixes_applied=0`）を維持。
- `auto_fix=false`:
  `linter.lint()` の結果に対し
  `passed = lint_passed(&issues, warnings_as_errors)`、
  `fixed_content = content.to_string()`、`fixes_applied = 0`、
  `converged = true`、`conflicting_rules = vec![]` を返す。
  - `converged=true` は lint-only 経路の番人値であり、
    **実測の収束ではない**（ADR-0111 / 0160 を継続）。

## 影響

- native / browser の判定ドリフトリスクを解消。
- `LintResultWrapper` の `#[derive(Debug, Clone, Serialize)]` は不変で、
  Tauri IPC 後方互換を維持。
- wasm は `converged` / `conflicting_rules` を
  `WasmLintFixResult` に公開しない（ADR-0160 の omit を維持）。
- `LintConfig.auto_fix` が権威、`WasmConfig.auto_fix` は未参照（既存）。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- 新規直接テスト 2 件:
  - auto_fix=false: 本文不変、`fixes_applied=0`、`converged=true`、
    競合空、`passed` 整合。
  - auto_fix=true: 全フィールドを `lint_and_fix()` と突き合わせ。
- `cargo test --all-targets --locked`: core **248** + desktop 62 + web 2。
- wasm-pack test --node: 10 件成功（auto_fix false / true 含む）。
- `cargo clippy --all-targets --locked -- -D warnings`、
  `./scripts/verify.sh`: 成功。E2E **250 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `converged` / `conflicting_rules` を JS へ additive getter 公開するか。
- lint-only 経路の `converged` を `Option` / 三値にするか。
- `js_sys::Error` 化と i18n、部分 config マージ／未知キー拒否。
- 反復上限 10 の根拠と `conflict_cycle`（ADR-0111 から継続）。
- 実 rumdl による非収束の E2E（現状は fake `FixResult` のみ、既存ギャップ）。
