---
type: Decision
title: "ADR-0122: 環境変数の max_retries マッピング"
description: "figment の split('_') で届かない max_retries を extract 後に手動で読み込む決定を記録する。"
tags:
  - ADR
  - config
  - security
generated:
  by: dsh
  at: "2026-11-18T07:45:00+09:00"
status: accepted
---

## 背景

`AppConfig::load()` は `Env::prefixed("OPENRUSTBOOKS_").split("_")` を使う。
figment 0.10.19 の `split("_")` は全ての `_` を `.` に置換するため、
`OPENRUSTBOOKS_AI_MAX_RETRIES` は `ai.max.retries` になり、
serde フィールド `ai.max_retries` に届かなかった（ADR-0120 / 0121 の未決定）。
`deny_unknown_fields` が無いためエラーにならず、黙って既定値 3 になる。

## 決定

- 純粋ヘルパー `apply_env_max_retries(&mut AppConfig, Option<String>)` を追加し、
  `AppConfig::load()` の `figment.extract()` 後に
  `std::env::var("OPENRUSTBOOKS_AI_MAX_RETRIES")` の値を渡す。
- 有効な `usize` のみ上書きし、未設定／不正値は既存値を維持する。
- figment の既存マッピング（`split("_")`）は変更しない
  （`OPENRUSTBOOKS_AI_ENABLED` 等、単一語フィールドを壊さないため）。
- 本修正は `max_retries` 1 フィールドのみ。秘密フィールド
  （`api_key` / `client_secret` / `refresh_token`）は対象外で、
  ADR-0098 の「env 経由のキー注入が効かない」という前提は不変。
- ヘルパーとテストは `#[cfg(not(target_arch = "wasm32"))]` で native 限定
  （`load()` 自体が native のみ）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| extract 後に 1 フィールド手動上書き（採用） | 既存 env を壊さず最小修正 |
| `split("_")` を `__` 等に変更 | 既存の単一語 env が一斉に壊れる |
| 秘密フィールドもマップ | ADR-0098 の前提が反転し、平文保存が顕在化する |

## 影響

- 優先順位は実質的に「有効な env > toml > default」。不正 env は未設定扱い。
- env 由来の `max_retries` は、その後の設定更新で `config.toml` に
  永続化され得る（非秘密値のため許容、ADR-0120 の保存値との乖離と同種）。
- 他フィールド（`default_provider` / `timeout_secs` / `base_url` 等）の
  アンダースコア問題は残る（今回の対象外）。
- 新規依存なし、unsafe なし。

## 検証

- 追加テスト 3 件（有効値 7 で上書き、不正値は維持、None は維持）。
- `cargo test --all-targets --locked`: core 173 + desktop 28 + web 2。
- wasm-test 3 件（native 限定テストは cfg で除外）。
- `./scripts/verify.sh` を実行し、6 ゲート・trunk build・WASM パス検査・
  E2E 228 項目がすべて成功した。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定（env-fix-auditor の指摘）

- 恒久的マッピング方式（手動上書き継続 vs figment `map()` による明示マップ）。
- 不正値ポリシーの統一（現状は黙視、figment 経由フィールドは
  extract エラー → 既定値フォールバックの可能性／推測）。
- env 由来値を `save()` で永続化するか。
- `default_provider` 等其他フィールドの対処範囲。
- ブラウザ側の env 相当（localStorage）の扱い。
