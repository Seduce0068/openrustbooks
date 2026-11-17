---
type: Decision
title: "ADR-0111: lint auto-fix の未収束 fail-closed"
description: "rumdl fix の未収束情報を保持し、未収束時は修正を採用しない決定を記録する。"
tags:
  - ADR
  - ai
  - lint
generated:
  by: dsh
  at: "2026-11-18T05:10:00+09:00"
status: accepted
---

## 背景

ADR-0110 で auto-fix を実装したが、`FixResult.converged` /
`conflicting_rules` / `conflict_cycle` を破棄し、`rules_fixed` だけを返していた。
未収束（ルール間の振動や反復上限到達）でも再 lint が通れば不安定な内容が
確定し得た（ADR-0110 の未決定、convergence-auditor の指摘）。

## 決定

- `LintResultWrapper` に `converged: bool` と `conflicting_rules: Vec<String>` を追加。
  - 値の意味: 非空 = 振動（ルール間循環）、空 = 反復上限到達などの別原因。
- 純粋ヘルパー `apply_fix_result(original, fixed, fix_result)` を追加する。
  - 収束時: （修正後本文, `rules_fixed`, true）
  - 未収束時: （元本文, 0, false）＝fail-closed
- `lint_and_fix()` は採用した本文（fail-closed 時は元本文）で再 lint し、
  `issues` と `passed` を作り直す。
  - `passed = converged && lint_passed`（未収束なら再 lint がクリーンでも必ず false）。
- `lint()` のみ／`auto_fix=false` 経路は `converged = true` /
  `conflicting_rules = vec![]`。
- Tauri IPC は Serialize の追加フィールドのみで後方互換。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 未収束時に元本文へ戻す（採用） | 安定性優先。途中結果を採用するより安全 |
| 上流 rumdl の通常 fix 経路（警告のみで修正採用） | 振動した内容が確定し得る |
| `converged=false` を無視 | 今回の問題が残る |

## 影響

- 振動／反復上限到達時は AI 生成結果も lint 修正も採用されず、
  `lint_passed = false` で返る（execute_pipeline は最終試行で Ok を返す既存挙動）。
- `fixes_applied = 0` は「修正を採用しなかった」という意味で、
  rumdl が実際に適用した回数ではない（ADR-0110 の注記を継承）。
- 新規依存は無い。

## 検証

- 新規テスト 2 件（収束時の採用、未収束時の fail-closed）と
  既存テスト 4 件の拡張（converged / conflicting_rules の検証）。
- `cargo test --all-targets --locked`: core 165 + desktop 28、すべて成功。
- `./scripts/verify.sh` を実行し、6 ゲート・trunk build・WASM パス検査・
  E2E 228 項目がすべて成功した。
- CI は billing issue で実行不可（ローカル検証で代替）。
- ついでに ADR-0110 の末尾に残っていたリテラル `EOF` を削除した。

## 未決定（convergence-auditor の指摘）

- 反復上限到達（conflicting_rules 空）と振動で同じ fail-closed にしているが、
  片方だけ「途中結果＋警告」にする選択肢もある。
- execute_pipeline の最終試行は `lint_passed=false` でも `Ok` を返すため、
  未収束が正常終了として UI に渡り得る。Err にするかは未決定。
- `conflict_cycle`（ルール順序）は Wrapper に含めていない。
- `WasmLintFixResult` は別構造体でスタブのまま（native との API 差）。
- 反復上限 10 の根拠再検証（上流 planning 経路は 100）。
