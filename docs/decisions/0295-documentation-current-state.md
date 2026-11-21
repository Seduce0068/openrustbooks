---
type: Decision
title: "ADR-0295: README / runbook / 索引の現行状態追従"
description: "実 WebView 検証済み、EPUB/ZIP 実装済み、E2E current 256 を入口文書へ反映する決定。"
tags:
  - ADR
  - documentation
generated:
  by: dsh
  at: "2026-11-22T05:12:00+09:00"
status: accepted
---

## 背景

README は実 WebView 未検証と EPUB / ZIP / Markdown を設計段階と記載していたが、
ADR-0282 / 0283 で実 WebView 検証と WDIO ゲートが完了し、
EPUB / ZIP / Markdown は実装済みだった。
runbook-verified-build は E2E current を 255 と記載していたが、
現行は Checker(expected=256) だった。

## 決定

- README の機能一覧を実装済み（EPUB / ZIP / Markdown）と
  設計段階（PDF / TXT）へ分離する。
- README の desktop 設定保存説明を ADR-0282 / 0283 の検証済み状態へ更新する。
- ADR-0273 は当時の環境ブロッカーの履歴として扱い、
  解消済みであることを追記する。
- runbook-verified-build の current E2E を 256（check 239 + equal 17）にする。
  255 は ADR-0265 の履歴値として保持する。
- README / docs/index に runbook-webdriver.md への導線を追加する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 入口文書を現行状態へ更新（採用） | 利用者とコントリビューターの誤認を防ぐ |
| ADR のみ更新 | README が古いままになる |
| 255 を現行扱いに戻す | Checker(expected=256) と矛盾するため却下 |

## 検証（Runtime Proof）

- markdownlint-cli2（全 docs / README / todo / GOAL）: 313 files / 0 issues。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- A05 履歴項目の古い表現を一体的に整理するか。
- 範囲外として残した README の build.rs / `--locked` 表記と ADR 要約誤配置。
