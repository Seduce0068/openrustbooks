---
type: Decision
title: "ADR-0341: CONFIG_CONFLICT 時の再読み込み導線"
description: "設定画面に get_config 再実行ボタンを追加し、競合時に view / revision を復帰可能にする決定。"
tags:
  - ADR
  - web
  - UX
generated:
  by: dsh
  at: "2026-11-23T02:49:00+09:00"
status: accepted
---

## 背景

CONFIG_CONFLICT の固定文言は「再読み込みしてからもう一度お試しください」と案内していたが、
設定画面には初回 `Effect::new` 以外の `fetch_config()` 呼び出しが無く、
画面内で再読み込みする手段が無かった。

## 決定

- 設定フォームに `settings-reload` ボタンを追加する。
- `fetch_config()` を再実行し、成功時に `config` / `revision` / `editable` を更新する。
- 失敗時は `config_fetch_status_message()` の固定文言のみを status へ出す。
- 生 message / code / path / 設定値は status へ出さない。
- 保存中（`saving=true`）は再読み込みを抑止する。

## 検証（Runtime Proof）

- `cargo fmt --all -- --check`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **263 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- 競合時に未保存編集を保持するか破棄するかの確認ダイアログ。
