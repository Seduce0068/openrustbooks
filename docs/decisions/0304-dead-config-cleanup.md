---
type: Decision
title: "ADR-0304: 未配線設定の DTO 除外と値保持"
description: "9 件の dead config を JS DTO から除外しつつ config.toml の値を保持し、未配線 UI を固定表示へ置き換える決定。"
tags:
  - ADR
  - config
  - web
generated:
  by: dsh
  at: "2026-11-22T07:55:00+09:00"
status: accepted
---

## 背景

設定 DTO / UI には動作へ反映されない dead フィールドが残っていた:
`general.telemetry`、`storage.max_recent`、`ai.max_tokens_per_month`、
`ai.model_overrides`、`ollama.keep_alive_minutes`、`lint.config_file`、
`storage.default_backend`、`editor.auto_save_delay`、`local_roots[].watch`。
利用者は設定可能と誤解していた。

## 決定

- 9 件すべてを JS DTO から除外する（配線は新機能のため今回の範囲外）。
- core の struct / serde 属性は変更せず、config.toml の読み書き・値保持を維持する。
- desktop の `into_config` はプレースホルダを入れ、
  `merge_secrets` が現在値から dead 値を復元する。
- `check_updates` の復元行・テスト、
  `model_overrides` / `config_file` の復元、
  local_roots の read_only / watch fail-closed は維持する。
- `OPENRUSTBOOKS_GENERAL_TELEMETRY` の env マップは無効化する。
- web の 6 箇所の form を「未配線」固定表示へ置換する。
- E2E mock_config から dead キーのみ削除し、
  check 名・件数・`Checker(expected=256)` は変更しない。

## 旧 ADR との関係

- ADR-0228 の「max_tokens_per_month の DTO 変更しない」は、
  DTO 除外と config.toml 値保持の両立へ更新する。
- ADR-0217 の editor / reader 値保持は維持し、
  auto_save_delay の DTO 除外のみを追加する。
- ADR-0277 の「DTO 完全削除の互換判断」は本 ADR で解消する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| DTO のみ除外 + 値保持（採用） | 誤解を防ぎつつ既存設定を失わない |
| `skip_serializing` で config.toml から削除 | 旧 ADR の値保持契約に反するため却下 |
| 全フィールドを配線 | 新機能実装で今回の範囲外 |

## 検証（Runtime Proof）

- core 331 / desktop 128 / web 6 tests、すべて成功。
- `cargo check --all-targets --locked`: rc=0。
- `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- wasm core / web check: rc=0。
- `npx tsc --noEmit`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- `local_roots[].watch` を id 非依存で現在値復元するか。
- 未配線 form の固定表示をさらに分かりやすくするか。
