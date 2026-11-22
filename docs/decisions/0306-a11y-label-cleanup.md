---
type: Decision
title: "ADR-0306: a11y 文言とリンク表示の改善"
description: "lang=ja に合わせた日本語化、リンク下線、file input focus、破壊操作の危険告知とライセンス導線を追加する決定。"
tags:
  - ADR
  - a11y
  - web
generated:
  by: dsh
  at: "2026-11-22T09:05:00+09:00"
status: accepted
---

## 背景

`lang="ja"` のページで主要ナビ / ボタン / Settings ラベルが英語のままだった。
リンクは色のみで識別され、隠し file input の focus が見えず、
Clear data の破壊性とライセンス全文への導線が不足していた。

## 決定

- Home / Settings / Reader / Editor の主要 nav / ボタン / 確認ダイアログ /
  Settings の h3 とラベルを日本語化する。
- E2E が厳密比較する以下は維持する:
  - h1 `Settings`
  - `#settings-language`
  - `更新確認: 未配線`
  - `設定を保存しました。`
  - `下書きを保存しました`
  - aria-label `Smaller text` / `Larger text` / `Chapter`
  - Cancel / Previous / dark mode / chapter filter / About の固定行
- リンク（`.app-header nav a` / `.reader-html a`）に下線を復帰する。
- 隠し file input は visually-hidden を維持し、
  `aria-describedby` + `:has(:focus-visible)` で可視ボタンへ focus 枠を出す。
- `Clear data` を「読書データを削除」へ変更し、
  confirm 文言へ「元に戻せません」を明記する。
- About に AGPL-3.0 公式 URL への「AGPL-3.0 全文」リンクを追加する。
- E2E が `window.confirm` を stub しているため、dialog 化は行わない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 可視文言 + CSS で改善（採用） | E2E 契約を壊さず WCAG 観点を改善できる |
| dialog 化と E2E 一括更新 | 256 件の check 名・文言を変えるため見送り |
| 英語ラベル維持 | lang=ja と不一致のため却下 |

## 検証（Runtime Proof）

- `cargo fmt --all -- --check`: rc=0。
- `cargo check -p openrustbooks-web --target wasm32-unknown-unknown --locked`: rc=0。
- `npx tsc --noEmit -p tsconfig.json`: rc=0。
- 静的ゲート 5 種（e2e-count / csp / updater / plugin / capability）: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- aria-label の Label in Name 対応（E2E 厳密比較の更新が必要）。
- 破壊操作の `<dialog>` 統一（E2E confirm stub の更新が必要）。
