---
type: Decision
title: "ADR-0120: AI リトライ回数の上限"
description: "max_retries を service 境界で 0..=10 にクランプし、暴走とオーバーフローを防ぐ決定を記録する。"
tags:
  - ADR
  - ai
  - config
generated:
  by: dsh
  at: "2026-11-18T07:25:00+09:00"
status: accepted
---

## 背景

`AiMarkdownService::new()` は `config.ai.max_retries` をそのまま
`PipelineConfig` に渡していた。大きすぎる値は API 呼び出しを最大 N+1 回まで
増やし（REQ-010 のコスト制御と緊張）、`usize::MAX` では
`max_retries + 1`（ADR-0118）がオーバーフローする。

## 決定

- `clamp_max_retries(value: usize) -> usize`（`value.min(10)`）を追加し、
  `AiMarkdownService::new()` で適用する。
- 0 は許可（`0..=0` で初期 1 回のみ、`attempts = 1` は ADR-0118 どおり）。
- 上限 10 は、ADR-0105:69 の既存の「例: 0..=10」に従った暫定値。
  REQ-010 の「最大 3 回再生成」の解釈差（初期 1 回＋リトライ N 回）は
  未解消のため、要件文言の明文化後に再評価する。
- クランプは **service 境界のみ**。保存済み `config.toml` の生値は
  書き換えない（ADR-0106 の DTO 除外方針と整合）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| service 境界でクランプ（採用） | 全実行経路を 1 箇所で守れる |
| config 保存時に正規化 | 既存設定を黙って書き換えることになり保留 |
| 上限なし | コスト暴走とオーバーフローが残る |

## 影響

- 実行時のリトライ回数は最大 10（＝最大 11 回生成）。
- `usize::MAX` でも 10 に丸められ、`max_retries + 1` のオーバーフローは
  service 経由では解消。
- 保存値 99 / 実行値 10 のような乖離は生じ得るが、UI には両方とも見えない
  （DTO 除外済み）。
- 新規依存なし。

## 検証

- 追加テスト 5 件（0 / 3 / 5 / 11 / usize::MAX）。
- `cargo test --all-targets --locked`: core 170 + desktop 28 + web 2、
  wasm-test 3、すべて成功。
- `./scripts/verify.sh` を実行し、6 ゲート・trunk build・WASM パス検査・
  E2E 228 項目がすべて成功した。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定（retry-limit-auditor の指摘）

- 保存値の正規化タイミング（load / merge / save）と、範囲外値を
  黙示丸めにするかエラー通知にするか。
- ~~REQ-010「最大 3 回」と `0..=max_retries` の解釈差~~ ADR-0121 で要件文言を明文化済み。
- 上限 10 の客観的根拠（実測コスト等）。
- `PipelineConfig` を直接生成する経路はクランプされない
  （pub struct / pub field）。型レベルでの制約は将来課題。
- `max_retries = 0` の pipeline 回帰テスト（`attempts == 1`）未追加
  （ADR-0118 から継続）。
- 環境変数 `OPENRUSTBOOKS_AI_MAX_RETRIES` が figment の `split("_")` で
  `ai.max.retries` に変換され、フィールドに届かない可能性（推測、未検証）。
