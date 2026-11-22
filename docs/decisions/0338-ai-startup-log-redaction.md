---
type: Decision
title: "ADR-0338: AI 初期化エラーのログ redaction"
description: "AI service 初期化失敗時に CoreError の Display をログへ出さず、code() 由来の情報だけを記録する決定。"
tags:
  - ADR
  - desktop
  - security
generated:
  by: dsh
  at: "2026-11-23T02:25:00+09:00"
status: accepted
---

## 背景

desktop/src/main.rs は起動時の AI 初期化失敗を
`log::error!("Failed to initialize AI service: {}", e)` で記録していた。
CoreError::Display は AiProvider の生エラー（host / response body / 認証情報）
を含むため、Stdout / LogDir / Webview の 3 ターゲットへ機微情報が残った。

## 決定

- 初期化失敗ログは `e.code()` 由来の固定情報だけを出力する。
- `CoreError::Display` を内部ログへも書かない。
- IPC redaction（ADR-0287）と Wasm redaction（ADR-0323）と同じ方針を適用する。

## 検証（Runtime Proof）

- `cargo fmt --all -- --check`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **263 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- AI service 初期化失敗を UI status へ伝播するか（現状はログのみ）。
