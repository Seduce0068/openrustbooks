---
type: Decision
title: "ADR-0329: EPUB2 NCX の Web E2E 追加"
description: "EPUB3 nav だけでなく EPUB2 NCX の目次読み込みと章移動を Web E2E で検証する決定。"
tags:
  - ADR
  - EPUB
  - E2E
generated:
  by: dsh
  at: "2026-11-23T00:08:00+09:00"
status: accepted
---

## 背景

core は OPF 2.0 + `application/x-dtbncx+xml` + `<spine toc="ncx">` の
NCX 目次をサポートしていたが、Web E2E は EPUB3 nav.xhtml のサンプルのみだった。
NCX フォールバックの回帰を自動ゲートで検出できなかった。

## 決定

- EPUB2 サンプル（OPF 2.0 + toc.ncx + ch1/ch2）を E2E fixture に追加する。
- check を 2 件追加する:
  - `EPUB2 NCX の目次が 2 件読み込まれる`
  - `NCX の項目選択で第 2 章へ移動する`
- `Checker(expected)` を 256 → 258 に更新する。
- `docs/e2e-traceability.md` の VC-004 と集計を 258 件へ更新する。
- `docs/documentation-operations.md` の current 値も 258 件へ追従する。
- 既存 EPUB3 check 名は変更しない。

## 検証（Runtime Proof）

- `python3 -m py_compile tools/verify-web-reader.py`: rc=0。
- check-e2e-count-consistency: `Checker(expected=258)` / 実数 258 一致。
- 隔離ポートでの E2E: **passed: 258, failed: 0 / all checks passed**。
- `./scripts/verify.sh`: 全ゲート成功、E2E **258 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- 追加 E2E を ADR-0006 の VC 表の別行へ分割するか（現状は VC-004 行に集約）。
