---
type: Decision
title: "ADR-0230: dark テーマのボタン文字コントラスト"
description: "dark のプライマリボタン文字色を暗色トークンへ変更し、WCAG AA を満たす決定を記録する。"
tags:
  - ADR
  - web
  - accessibility
generated:
  by: dsh
  at: "2026-11-19T17:18:00+09:00"
status: accepted
---

## 背景

a11y 監査で、light の muted / dark の muted は WCAG AA を満たす一方、
dark の `.btn-primary` は白文字 / `#3b82f6` で **3.68:1**、
hover の `#60a5fa` では **2.54:1** となり
WCAG 1.4.3（通常テキスト 4.5:1）未達と判定された（R6 B07）。
`.btn-danger` は白文字 / `#dc2626` で 4.83:1 と既に合格だった。

## 決定

- `--color-on-accent` を追加する。
  - `:root`: `#ffffff`
  - `@media (prefers-color-scheme: dark) :root`: `#0f172a`
  - `html.dark`: `#0f172a`
- `.btn-primary` の文字色を `var(--color-on-accent)` に変更する。
- `.btn-danger` は `color: white` を維持する
  （暗色文字にすると `#dc2626` 上で 3.70:1 に低下するため）。
- hover 背景色は変更しない。
- light テーマの外観は変更しない。
- E2E は色値へ依存せず、期待総数 254 を維持する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| primary のみ暗色文字（採用） | 両テーマで AA を満たし最小差分 |
| primary / danger 両方を暗色文字 | danger が 3.70:1 に低下するため却下 |
| danger 背景を明るい赤へ変更 | デザイン変更が大きく本次範囲外 |

## 影響

- dark の primary 通常: **4.85:1**、hover: **7.02:1**（いずれも AA 合格）。
- dark の danger: 4.83:1 を維持。
- light の primary: 5.17:1 / hover 6.70:1 を維持。
- E2E の色依存チェック（toolbar 背景 / transition）は影響外。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- WCAG 相対輝度による静的計算で
  dark primary 通常 / hover、dark danger、light primary / hover を確認。
- `grep`: E2E に `btn-primary` / `btn-danger` / 計算済み文字色への依存なし。
- `./scripts/verify.sh`: 全ゲート成功、E2E **254 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 枠線のコントラスト（light `#e5e7eb` / 白 = 1.24:1、
  dark `#334155` / `#0f172a` = 1.72:1）は WCAG 1.4.11 の観点で
  解釈が必要なため本次据え置き。
- `prefers-contrast: more` の対応（ADR-0039 の未決定を継続）。
- 実ブラウザ / 支援技術での目視・実測（静的計算のみ）。
- `.btn-secondary`、focus リング、他コントロールの包括的なコントラスト監査。
