---
type: Decision
title: "ADR-0330: prefers-contrast: more の動的 E2E"
description: "高コントラスト設定の CSS を CDP emulation と WCAG コントラスト比で検証する決定。"
tags:
  - ADR
  - E2E
  - a11y
generated:
  by: dsh
  at: "2026-11-23T00:24:00+09:00"
status: accepted
---

## 背景

web/index.html には `@media (prefers-contrast: more)` の CSS があったが、
ADR-0236 は静的なコントラスト比再計算のみで E2E 検証が無かった。
スタイル変更や cascade 順序の誤りを自動ゲートで検出できなかった。

## 決定

- CDP `Emulation.setEmulatedMedia` で `prefers-contrast: more` を設定する。
- `matchMedia` が有効であることを検証する。
- light / dark それぞれで `--color-bg` / `--color-border` / `--color-muted` の
  emulation 前後の変化と WCAG 比（枠線 3:1 / 補助文字 4.5:1）を検証する。
- 色リテラル固定比較は使わず、実行時の computed style から比を算出する。
- emulation は try/finally で `features=[]` へ復元する。
- 5 check を追加し `Checker(expected)` を 258 → 263 に更新する。
- e2e-traceability.md / documentation-operations.md も 263 件へ追従する。
- ADR-0236 の「prefers-contrast E2E なし」という記述は本 ADR が更新する。

## 検証（Runtime Proof）

- `python3 -m py_compile tools/verify-web-reader.py`: rc=0。
- check-e2e-count-consistency: `Checker(expected=263)` / 実数 263 一致。
- 隔離ポート E2E: **passed: 263, failed: 0 / all checks passed**。
- `./scripts/verify.sh`: 全ゲート成功、E2E **263 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- 既存のダーク時ツールバー背景 check のリテラル比較を比ベースへ置き換えるか。
- dark 経路でも matchMedia check を明示的に追加するか。
