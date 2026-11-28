---
type: Decision
title: "ADR-0360: Wasm lint DTO / import 告知 / JS エラー型の設計判断"
description: "残る 3 件の設計判断を確定する。lint 収束情報を公開し、import 復元オプションを未配線告知し、JS エラーは文字列を維持する。"
tags:
  - ADR
  - wasm
  - web
generated:
  by: dsh (lead)
  at: "2026-11-28T17:29:21+09:00"
status: accepted
---

## 決定 1: lint 収束情報の公開

`WasmLintFixResult` へ `converged` / `conflicting_rules` と getter を追加し、
native `LintResultWrapper` の値をそのまま渡す。ADR-0160 / ADR-0194 の「公開しない / 未決定」は
本 ADR が上書きする。判定の権威は `validate_with_auto_fix` のままで、fail-closed 契約は不変。

## 決定 2: 設定 import の明示復元オプション

UI を新設せず、reader import コントロール直後に固定日本語の未配線告知を表示する。
現在値保護の契約と import/export 互換は変更しない。動的 status の現在値保護文言は維持する。

## 決定 3: wasm JS エラー型

`JsValue::from_str(fixed_message(code()))` を維持し、`js_sys::Error` へ移行しない。
利用側は固定日本語 status のみを表示し `e.message` を読む実コードが無い。移行は wasm test の
`as_string()` 契約（5 箇所）を壊し、情報量も増えない。ADR-0156 の未決定は本 ADR で決定済みとする。

## 検証（Runtime Proof）

- `cargo fmt --all -- --check` / `cargo check --all-targets --locked` / `cargo test --all-targets --locked`
  （35 passed）/ `cargo clippy --all-targets --locked -- -D warnings` / wasm ターゲット check: rc=0。
- `bash scripts/ci-wasm-test.sh`: rc=0（20 passed）。
- `cargo audit`: 新規 `RUSTSEC-2026-0594`（cookie 0.18.2）を検知したため、`cargo update -p cookie --precise 0.18.3`
  で修正し rc=0。allowlist 追加ではなく依存更新を選択した。
- `bash scripts/verify.sh`: 実行結果はコミット前に確認する。

## 未決定

- 未収束ケースを wasm の `lint_and_fix` 経路から安定して作る test 注入手法（ADR-0194 を継承）。
- 構造化 `code` を含む JS エラー公開が必要になった場合の別設計。
