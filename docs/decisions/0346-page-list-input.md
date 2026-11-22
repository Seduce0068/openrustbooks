---
type: Decision
title: "ADR-0346: Page-list 入力の使いやすさ改善"
description: "ページ入力に候補表示・Enter 移動・trim/大小文字無視照合を追加する決定。"
tags:
  - ADR
  - web
  - UX
generated:
  by: dsh
  at: "2026-11-23T03:52:00+09:00"
status: accepted
---

## 背景

ページ移動入力は page_list タイトルとの完全一致が必要で、
前後空白や大小文字の違いで失敗した。
候補表示も Enter 移動も無く、キーボード操作では「移動」ボタンへ移る必要があった。

## 決定

- `<datalist id="reader-page-options">` に page_list タイトルを列挙し、
  入力へ `list` 属性を付与する。
- Enter キーで移動を発火する。
- 照合は `trim().to_lowercase()` で行う。
- 未検出時の status は既存 E2E 契約（数字のページ番号）を維持するが、
  数字以外の入力はエコーせず固定文言「ページが見つかりません。」を出す。
- E2E check 名 / `Checker(expected=263)` は変更しない。

## 検証（Runtime Proof）

- `cargo fmt --all -- --check`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **263 passed / 0 failed**、WDIO gate rc=0。
  - 「存在しないページ番号では状態メッセージが出る」は数字エコー维持で通过。

## 未決定

- 部分一致 / あいまい検索の導入。
