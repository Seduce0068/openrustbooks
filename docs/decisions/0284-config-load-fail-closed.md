---
type: Decision
title: "ADR-0284: config.toml 読み込み失敗時の fail-closed 化"
description: "設定読み込みエラーを握り潰して既定値で上書きする経路を廃し、退避と保存拒否を義務付ける決定。"
tags:
  - ADR
  - config
  - security
generated:
  by: dsh
  at: "2026-11-22T01:20:00+09:00"
status: accepted
---

## 背景

`desktop/src/main.rs` は `AppConfig::load().unwrap_or_default()` で読み込みエラーを捨てていた。
その後の `commit_config` が `config.save()` を呼ぶため、不正 TOML や権限エラーで
読み込めなかった `config.toml` が既定値で上書きされ、API key / OAuth token /
local_roots が失われる危険があった。

## 決定

- 読み込み失敗時は生エラー文字列と絶対パスをログへ出さず、
  固定エラー code と固定文言のみを報告する。
- 元 `config.toml` を `config.toml.failed-<timestamp>-<counter>` へ退避する。
- 退避ファイルは Unix で owner-only (0600) に制限する。
- `AppState` に `config_load_failed` を保持し、
  `commit_config` と全 config 保存 command を fail-closed で拒否する。
- `AppConfig::load()` / `save()` / 退避処理は
  `config_dir_path()` / `config_file_path()` を単一情報源として使う。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 退避 + 保存拒否（採用） | 元データを保持しつつ誤上書きを防げる |
| 既定値で起動してそのまま保存 | 既存設定を破壊するため却下 |
| 読み込み失敗時に起動停止 | 利用者が復旧できないため今回は採用しない |

## 影響

- 読み込み失敗後の config 保存は
  `CONFIG_LOAD_FAILED` の固定文言で拒否される。
- 2 回目以降の起動では退避済みのため load が成功し、
  既定値ベースの新規 config へ保存できる（ADR の未決定で追跡）。
- 退避先の保持期間・世代数は未定義。
- パス・生エラー・設定値はログへ出ない。

## 検証（Runtime Proof）

- `cargo test -p openrustbooks-core --locked`: 293 passed / 0 failed。
- `cargo test -p openrustbooks-desktop --locked`: 92 passed / 0 failed。
- 追加 test:
  - `backup_moves_original_and_uses_failed_suffix`
  - `backup_is_noop_when_original_is_absent`
  - `backup_names_do_not_collide`
  - `config_path_helpers_point_to_same_file`
  - `backup_restricts_permissions_to_owner_only`
  - `invalid_toml_file_fails_the_load_path`
  - `commit_config_rejected_when_config_load_failed`
- `cargo fmt --all -- --check`: rc=0。
- `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- 退避ファイルの保持期間・最大世代数・UI 通知。
- 退避失敗時に起動を停止するか。
- 読み込み失敗状態を次回起動へ永続化するか。
- 非 Unix での退避ファイル権限制御。
