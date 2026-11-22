---
type: Decision
title: "ADR-0347: ネイティブ dialog の共通スタイル"
description: "削除確認とヘルプの dialog に共通スタイルと backdrop / forced-colors 対応を適用する決定。"
tags:
  - ADR
  - CSS
  - a11y
generated:
  by: dsh
  at: "2026-11-23T03:58:00+09:00"
status: accepted
---

## 背景

削除確認ダイアログには theme / border / backdrop のスタイルがあったが、
ヘルプダイアログには無く、dark 表示や forced-colors で既定の明るい外観のまま浮いた。

## 決定

- `dialog#reader-delete-dialog` 固有のスタイルを `dialog[open]` の共通スタイルへ変更し、
  ヘルプダイアログにも適用する。
- `dialog::backdrop` に背景・`overflow: hidden`・`overscroll-behavior: contain` を設定する。
- `dialog[open]` にも `overscroll-behavior: contain` を設定し、
  backdrop 越しのページスクロールを防ぐ。
- 既存の forced-colors トークン上書きはそのまま適用する。
- `display` は無条件に指定せず、closed 状態を壊さない。

## 検証（Runtime Proof）

- `./scripts/verify.sh`: 全ゲート成功、E2E **263 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- ヘルプダイアログの dark / forced-colors 表示を E2E で画像比較するか。
