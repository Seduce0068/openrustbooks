---
type: Decision
title: "ADR-0331: get_config 失敗 status の区別"
description: "ブラウザ非対応と desktop IPC 失敗を別の固定文言で表示する決定。"
tags:
  - ADR
  - web
  - IPC
generated:
  by: dsh
  at: "2026-11-23T00:50:00+09:00"
status: accepted
---

## 背景

fetch_config() は IPC 失敗を単なる String に落とし、
SettingsForm は全エラーを「デスクトップ版のみ利用可能」と表示していた。
desktop で config.toml 読み込みに失敗した場合も同じ案内になり、
利用者は原因を判断できなかった。

## 決定

- fetch_config() を `Result<SettingsView, CommandError>` に変更する。
- `invoke_update("get_config", ...)` を使い、reject から code のみを保持する。
- deserialize 失敗は `CONFIG_INTERNAL` に落とす。
- `config_fetch_status_message(code)` を追加する:
  - `desktop only` → 既存「デスクトップ版のみ利用可能」
  - それ以外 → 「設定を取得できませんでした。config.toml を確認してください。」
- 生 message / code / path / 設定値は status へ出さない。
- 旧 `invoke()` は不要になったため削除する。

## 却下した方針

- WDIO で `core.invoke` を差し替えて reject を注入する test は、
  tauri-service が独自コンテキストで invoke を解決するため競合し、
  `core.invoke not available after 5s timeout` を連発して不安定化した。
  採用せず、`config_fetch_status_tests` の単体 test で代替する。

## 検証（Runtime Proof）

- web test: **28 passed / 0 failed**（新規 2 件含む）。
- `cargo fmt --all -- --check`: rc=0。
- wasm web check / `npx tsc --noEmit`: rc=0。
- check-e2e-count-consistency: expected=263 / 実数 263。
- `npm run test:wdio`: 1 passing。
- `./scripts/verify.sh`: 全ゲート成功、E2E **263 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- 取得失敗時に再読み込みボタンを提供するか。
