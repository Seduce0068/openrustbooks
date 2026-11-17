---
type: Decision
title: "ADR-0105: AI リトライ回数の専用設定"
description: "リトライ回数をトークン上限から分離し、AiConfig.max_retries に集約する決定を記録する。"
tags:
  - ADR
  - ai
  - config
generated:
  by: dsh
  at: "2026-11-18T03:50:00+09:00"
status: accepted
---

## 背景

`core/src/ai_markdown/pipeline.rs:27` は
`max_retries: config.ai.max_tokens_per_request.min(3)` としており、
トークン上限（8192 等）をリトライ回数（最大 3）に流用していた。
設定を変更するとリトライ回数が意図せず変わる問題があった
（todo.md R1-ai#3 / R2-ai#3）。

## 決定

- `AiConfig` に `pub max_retries: usize` を追加し、既定値を 3 にする。
- `#[serde(default = "default_max_retries")]` を付け、キー欠落時も 3 となる
  後方互換を確保する。
- `PipelineConfig.max_retries` は `config.ai.max_retries` を使う。
- `AiConfigView` に `max_retries` を含め、JS から設定可能にする。
- トークン上限 `token_limit` は従来どおり `max_tokens_per_request` を使う。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 専用フィールドを追加（採用） | 意味が明確で、設定変更の副作用がない |
| 既存フィールドから導出 | 今回のバグの原因 |
| ハードコード 3 | 利用者が調整できず REQ-010 のコスト制御と矛盾 |

## 影響

- 旧 `config.toml` に `max_retries` が無くても既定 3 で読み込める。
- 次回保存で `config.toml` に `[ai] max_retries` が書き出される。
- IPC の `get_config` レスポンスに `ai.max_retries` が追加される。
- Wasm 側（`WasmConfig`）は `AiConfig` を個別コピーしていないため影響なし
  （retry-impact-auditor の確認）。

## 検証

- 追加テスト 4 件（既定値、キー欠落時の既定復元、明示値 5 の復元、
  DTO JSON への含有）。
- `cargo test --all-targets --locked`: core 151 + desktop 22、すべて成功。
- `./scripts/verify.sh` を実行し、6 ゲート・trunk build・WASM パス検査・
  E2E 228 項目がすべて成功した。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 解決済み

- 旧 DTO JSON（キーなし）を `update_config` へ渡すと 3 で上書きされる問題は
  ADR-0106 で `max_retries` を DTO から除外しマージ保持する方式に解決済み。
- `update_config` が AI 設定変更時に service を再初期化せず、実行中 pipeline に
  新値が反映されない問題は ADR-0107 で fail-closed な再初期化に解決済み。

## 未決定（retry-impact-auditor の指摘）

- ~~REQ-010 の「最大 3 回再生成」と実装の解釈差~~ ADR-0121 で要件文言を明文化済み。
- ~~`max_retries` の範囲バリデーション（例 0..=10）未実装~~ ADR-0120 で service 境界にクランプを実装済み。
- ~~`WasmConfig::default().ai_enabled = true` が `AiConfig` の既定 false と
  矛盾~~ ADR-0117 で解消済み。
EOF
