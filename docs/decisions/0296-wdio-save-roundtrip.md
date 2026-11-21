---
type: Decision
title: "ADR-0296: WDIO 実 WebView 設定保存往復の検証"
description: "WebdriverIO の settings spec に update_config_patch の revision +1 と config.toml 永続化を追加する決定。"
tags:
  - ADR
  - WDIO
  - E2E
generated:
  by: dsh
  at: "2026-11-22T05:27:00+09:00"
status: accepted
---

## 背景

ADR-0282 は実 WebView probe で update_config_patch の成功、
revision +1、config.toml 永続化を確認した。
しかし ADR-0283 の WDIO ゲートは get_config 読み取りのみで、
その保存往復を回帰検証していなかった。

## 決定

- e2e/specs/settings.e2e.ts に設定保存往復を追加する。
- `window.__TAURI__.core.invoke('update_config_patch', { patch: ... })` を呼び、
  revision +1 を検証する。
- 非秘密キー `log_level` を一時変更し、config.toml の `[general]` セクション
  から値を読み取って永続化を検証する。
- テスト終了時（失敗時も）は最新 revision を再取得して元の log_level へ復元する。
- 読み取り例外には絶対パスを含めず固定文言のみを使う。
- 既存 spec 名 / it 名 / CDP mock E2E 256 件・check 名は変更しない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| WDIO に保存往復を追加（採用） | 実 WebView 固有の IPC 引数不具合を回帰検証できる |
| 手動 probe のみ | 再発を検出できない |
| CDP mock に依存 | ADR-0282 の通り実 WebView の代替にならない |

## 検証（Runtime Proof）

- `npx tsc -p tsconfig.json`: rc=0。
- `npm run test:wdio`: **1 passing**（1 分以内）。
- config.toml に `log_level = "debug"` が書き込まれ、
  finally で元値 `info` へ復元されたことを確認。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- revision は保存時に進み続けるため、テスト間で状態を完全には復元しない。
- 別の非秘密キーを用いた保存往復の追加。
