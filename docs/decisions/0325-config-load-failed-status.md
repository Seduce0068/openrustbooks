---
type: Decision
title: "ADR-0325: CONFIG_LOAD_FAILED の web 固定文言"
description: "desktop が返す CONFIG_LOAD_FAILED code を web の status で専用文言へ表示する決定。"
tags:
  - ADR
  - web
  - IPC
generated:
  by: dsh
  at: "2026-11-22T21:54:00+09:00"
status: accepted
---

## 背景

desktop は config.toml 読み込み失敗時に `CONFIG_LOAD_FAILED` code を返すが、
web の `CommandError::status_message` は未対応で、
既定の「設定を保存できませんでした。」へ落としていた。
保存が無効な理由が利用者に伝わらなかった。

## 決定

- `CONFIG_LOAD_FAILED` に固定日本語文言を割り当てる:
  `設定ファイルを読み込めませんでした。読み込み失敗のため保存は無効です。config.toml を確認してください。`
- code 文字列 / 完全パス / 設定値 / 生 message は文言へ出さない。
- desktop が返す保存系 code 5 種
  （CONFIG_CONFLICT / CONFIG_VALIDATION / CONFIG_STORAGE /
  CONFIG_LOAD_FAILED / AI_PROVIDER）すべてが専用文言を持つことを test で固定する。
- 未知 code の安全な既定文言は維持する。

## R4 指摘の採用・却下

- `WINDOW_INTERNAL` は desktop の保存系 CommandError 経路で返らず、
  web の status 対象外のため、専用文言を追加しない（却下）。
- code 集合の手書き test 配列は、保存系 code に限定した明示的な一覧として受容する。

## 検証（Runtime Proof）

- web test: **20 passed / 0 failed**。
- `cargo fmt --all -- --check`: rc=0。
- `cargo check -p openrustbooks-web --target wasm32-unknown-unknown --locked`: rc=0。
- `npx tsc --noEmit`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- 保存系 code 集合を core / desktop の共有定数と自動同期する仕組み。
