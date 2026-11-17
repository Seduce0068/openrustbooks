---
type: Decision
title: "ADR-0118: AI pipeline のデッドコード削除"
description: "execute_pipeline の到達不能な last_error と動的 Err を削除する決定を記録する。"
tags:
  - ADR
  - ai
  - refactor
generated:
  by: dsh
  at: "2026-11-18T06:50:00+09:00"
status: accepted
---

## 背景

`execute_pipeline()` は `lint_result.passed || attempt == max_retries` で
最終試行に必ず `Ok` を返すため、`last_error` とループ後の動的 `Err` は
到達不能だった（ADR-0116 の未決定）。

## 決定

- `let mut last_error = None;`、ループ内の代入、ループ後の動的 `Err` を削除する。
- ループ後には型チェックを満たすための静的
  `Err(CoreError::ai_pipeline("Pipeline failed after max retries"))` を残す
  （旧文言を維持）。`panic!` / `unreachable!` / unsafe は使用しない。
- 最終 lint 不合格時に `Ok` ＋ `lint_passed=false` を返す ADR-0116 の決定は
  **変更しない**。
- `call_ai()` / `validate_content()` の `?` によるエラー伝播も不変。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 静的 Err を残す（採用） | 最小差分・panic なし・挙動不変 |
| `loop` に変更して到達不能分岐を消す | 差分が大きく回帰リスクが上がる |
| `unreachable!()` | AGENTS.md §15 の禁止事項に反する |

## 到達性の根拠（pipeline-auditor の監査）

- `max_retries: usize`（最小 0）に対し `0..=max_retries` は必ず非空。
- 最終 `attempt` は必ず `max_retries` に等しい。
- `||` の短絡により最終試行で条件が必ず真 → `return Ok`。
- `self.config` に内部可変性が無く、範囲端点と比較値は同一。

## 影響

- 観測可能な挙動は不変（`max_retries = 0` でも 1 回実行され `Ok(attempts=1)`）。
- 新規依存なし。

## 検証

- `cargo fmt --all` / `cargo check --all-targets --locked` / `cargo test --all-targets --locked`
  / `cargo clippy --all-targets --locked -- -D warnings` すべて成功。
- `./scripts/verify.sh` を実行し、6 ゲート・trunk build・WASM パス検査・
  E2E 228 項目がすべて成功した。
- 全単体テスト core 165 + desktop 28 + web 2。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `max_retries = 0` の回帰テストは未追加（監査で推奨）。
- `max_retries + 1` / `attempt + 1` が `usize::MAX` でオーバーフローする
  既存課題（今回のスコープ外）。
