---
type: Decision
title: "ADR-0269: 設定保存 IPC の UI 起点 CAS と構造化エラー"
description: "update_config_patch に UI が読んだ revision を必須で渡し、衝突を構造化エラーで判別可能にする決定。"
tags:
  - ADR
  - ipc
  - config
generated:
  by: dsh
  at: "2026-11-20T04:54:00+09:00"
status: accepted
---

## 背景

ADR-0210 の CAS はコマンドが内部で取得した revision を使い、
UI が `get_config` で読んだ時点との比較ではなかった。
そのため stale な画面内容が別更新を上書きし得た。
また設定コマンドは `Result<T, String>` で、
Conflict / Validation / Storage / Internal を機械的に判別できなかった。
ADR-0253 の実装前提 3 はこの 2 点の解消を求めていた。

## 決定

- `core::CoreError::code()` を追加し、エラー種別を
  `CONFIG_CONFLICT` / `CONFIG_VALIDATION` / `CONFIG_STORAGE` /
  `CONFIG_INTERNAL` に固定する（Display / 既存挙動は不変）。
- `desktop::commands::config::CommandError { code, message }` を追加する。
  - `message` は code ごとの固定文言のみ。
  - 生エラー / パス / 設定値 / 秘密情報を出さない。
- `ConfigPatch` に必須 `expected: u64` を追加する（serde default 禁止）。
  `ConfigView` のフィールド名は `revision` のまま（入出力の非対称は意図）。
- `update_config_patch` は
  `Result<ConfigView, CommandError>` を返し、
  `patch.expected` を `commit_validated_config` へ渡す。
  `current` の内部取得は `merge_secrets` 用に留める。
- `update_config` / `reset_config` / `clear_secrets` は
  既存の `Result<T, String>` を維持する（ADR-0253 の最小 IPC 外）。
- web / E2E / IPC 経路（ADR-0264）は変更しない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 既存 update_config_patch を拡張（採用） | 新_command を増やさず ADR-0253 の最小 IPC を維持 |
| 新規 ConfigUpdateRequest command | invoke_handler / capability の再設計が必要 |
| String エラーのまま文言判別 | ローカライズ・文言変更で壊れる |

## 影響

- ConfigPatch を deserialize する呼び出しは `expected` が必須になる。
- 古い expected を渡すと `CONFIG_CONFLICT` で失败し、
  設定 / revision / service / disk は不変。
- 成功時は `ConfigView.revision` が expected + 1 になる。
- E2E check 名・件数（255）は不変。

## 検証（Runtime Proof）

- core error code test 3 件追加、desktop config test 4 件追加 + 5 件更新。
- `cargo test --all-targets --locked`: core **287** / desktop **82** / web 2。
- `wasm-pack test --node --locked`: 14 passed。
- `cargo clippy --all-targets --locked -- -D warnings`、
  wasm32 core / web check、`./scripts/verify.sh`: すべて成功。
- E2E **255 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `update_config` / `reset_config` / `clear_secrets` の構造化エラー移行時期。
- web 側 reject 改修（CommandError の code 判別）は ADR-0264 の IPC 経路決定後。
- `CommandError.message` の固定文言一覧とローカライズ方針。
- reset / clear の UI 接続と権限分離（ADR-0134 / 0135）。
- プロセス外で config.toml を編集した場合の revision 検出。
- revision 溢出 / pending 回復（ADR-0210 を継続）。
