---
type: Decision
title: "ADR-0133: max_retries=0 の回帰テスト"
description: "初期1回のみ実行される挙動をテストで固定し、ADR-0118/0120 の未決定を解消する。"
tags:
  - ADR
  - ai
  - testing
generated:
  by: dsh
  at: "2026-11-18T11:10:00+09:00"
status: accepted
---

## 背景

`execute_pipeline()` は `for attempt in 0..=max_retries` で、
`max_retries = 0` なら初期 1 回のみ実行され `attempts = 1` になる。
これまで専用テストが無く、ADR-0118 / 0120 で未決定として残っていた。

## 決定

- `core/src/ai_markdown/mod.rs` の `#[cfg(test)] mod tests` に 2 件追加
  （本番コード・新規依存なし）。
  1. `zero_max_retries_runs_once_and_reports_lint_pass`: lint 合格ケース。
  2. `zero_max_retries_keeps_ok_when_lint_fails`: 主管帰テスト。
- 主管帰テストのオラクル（3 つ併用）:
  - モック `complete()` の呼び出し回数 == 1（インスタンス別 `AtomicUsize`、
    static は並列テスト汚染のため不使用）
  - `result.attempts == 1`
  - `result.is_ok()` かつ `!result.lint_passed`（最終 attempt バイパス＝ADR-0116）
- fixture: `PipelineConfig { max_retries: 0, auto_fix: false, ..Default::default() }`
  ＋既存 `strict_linter()`（`warnings_as_errors: true`）＋モック応答
  `<div>html</div>`（MD033 の修正不能 Warning）。
  末尾空白（DIRTY）は auto_fix / Warning 無視で合格し得るため失敗 fixture に使わない。
- モック: `#[async_trait]` 付きで `complete` / `stream_complete` /
  `count_tokens` / `model_name` の 4 メソッドを実装。
  `AiMarkdownPipeline` はフィールド直接構築、
  `PromptRegistry::with_builtins()` を使用（`load()` / `global_registry()` は不使用）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 3 オラクル併用（採用） | lint 合格だけでは「リトライ抑止」を証明できない |
| `is_ok()` のみ | ループが余分に回っても検出できない |
| DIRTY を失敗 fixture に | auto_fix / Warning 無視で合格し得る |

## 影響

- `pipeline.rs:290-293` の `clamp_zero_stays_zero`（service 境界）とは
  別レイヤーのテストであり重複ではない。
- 本番コード変更なし。

## 検証

- `cargo test --all-targets --locked`: core 188 + desktop 44 + web 2、
  wasm-test 3、audit 脆弱性 0 件、E2E 229 項目すべて成功。
- E2E は途中 2 回フレーク（設定画面と画像警告）したが、
  3 回連続成功＋verify.sh 再実行で解消（本変更とは無関係）。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定（zero-retry-auditor の指摘）

- `PipelineConfig` 直接生成経路は `clamp_max_retries` を通らない
  （ADR-0120 から継続。本テストは意図的に直接生成）。
- 直接経路での `usize::MAX` オーバーフロー（ADR-0118 から継続）。
- `max_retries=3` で lint 失敗時に `attempts==4` となる対照テスト。
- rumdl アップグレード時に fixtures が不合格を維持するかの再検証。
