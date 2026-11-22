---
type: Decision
title: "ADR-0343: AI 送信確認プレビューの末尾表示"
description: "送信確認ダイアログで先頭だけでなく末尾も抜粋し、省略量と全文確認を明示する決定。"
tags:
  - ADR
  - AI
  - security
generated:
  by: dsh
  at: "2026-11-23T03:18:00+09:00"
status: accepted
---

## 背景

AI 送信確認の要約は各フィールドの先頭 80 文字だけを表示していた。
末尾に個人情報や社内情報がある場合、利用者は内容を確認したつもりで
そのまま外部 AI プロバイダへ送信してしまった。

## 決定

- 各フィールドの先頭 80 文字と末尾 80 文字を表示する。
- 省略がある場合は省略文字数を明示する。
- ダイアログ冒頭に「表示は抜粋です。送信前に全文を確認してください。」を固定文言で追加する。
- 値・host・body を新たにダイアログやログへ出すわけではなく、表示対象は送信対象の本文のみ。

## 検証（Runtime Proof）

- `summary_truncates_long_multibyte_value_to_80_chars` を 200 文字ケースへ更新し、
  先頭 80 / 末尾 80 / 省略 40 chars / 全文確認文言を assert。
- `cargo fmt --all -- --check`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **263 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- 全文をスクロール可能なダイアログで表示するか（現状は抜粋＋明示的な確認促し）。
