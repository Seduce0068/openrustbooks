---
type: Decision
title: "ADR-0293: lint feedback が no-op な要求のリトライ停止"
description: "Table / FrontMatter の lint 不合格時に同一 prompt の無駄な AI 再呼び出しを行わない決定。"
tags:
  - ADR
  - AI
  - lint
generated:
  by: dsh
  at: "2026-11-22T04:38:00+09:00"
status: accepted
---

## 背景

TableRequest / FrontMatterRequest の add_lint_feedback は空実装だった。
pipeline は lint 不合格のたびに同一 prompt / content で AI を再呼び出しし、
最大 11 回の外部呼び出しが無駄に発生した。

## 決定

- PipelineInput に `supports_lint_feedback()` を追加する（既定 true）。
- Table / FrontMatter は false を返す。
- execute_pipeline は lint 不合格時に
  `supports_lint_feedback()==false` なら即座に `lint_passed: false` で返す。
- Generate / Refine は従来どおり feedback を反映して retry する。
- execute_pipeline 内でも `max_retries` を 10 以下へ clamp し、
  `usize::MAX` でも無限ループ / オーバーフローしないようにする。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| no-op 要求は即時停止（採用） | 無駄な課金とレート制限消費を防げる |
| 全要求でretry継続 | prompt が変わらず意味がない |
| add_lint_feedback を全要求に実装 | 今回の範囲外 |

## 検証（Runtime Proof）

- 新規 test 6 件:
  - feedback 対応判定、Table / FrontMatter の 1 回停止、
    Generate / Refine の従来 retry、oversized max_retries の clamp。
- `cargo test -p openrustbooks-core --all-targets --locked`: **318 passed / 0 failed**。
- `cargo fmt --all -- --check`: rc=0。
- `cargo clippy -p openrustbooks-core --all-targets --locked -- -D warnings`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- add_lint_feedback の戻り値で反映有無を判定する設計への統合。
- Refine の retry に auto-fix 済み本文を反映する（ADR-0116）。
- ADR-0133 の対照テスト 기록の追従。
