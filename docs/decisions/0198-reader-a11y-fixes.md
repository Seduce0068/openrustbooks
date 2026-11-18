---
type: Decision
title: "ADR-0198: Reader の ARIA 属性修正"
description: "本文領域のアクセシブル名を aria-labelledby へ一本化し、ダーク切替の状態を実効値で告知する決定を記録する。"
tags:
  - ADR
  - web
  - accessibility
generated:
  by: dsh
  at: "2026-11-19T08:39:00+09:00"
status: accepted
---

## 背景

- `.reader-content` に `aria-label="Chapter content"` と
  `aria-labelledby="reader-chapter-title"` が併記されていた。
  ARIA では `aria-labelledby` が優先され、`aria-label` は冗長だった
  （R6 PR-L01）。
- `#reader-theme` に `aria-pressed` が無く、隣接する
  Auto / Downscale トグルと状態告知の挙動が不一致だった（R6 PR-L02）。

modern-web-guidance の accessibility では
「Prefer aria-labelledby over aria-label when a visible label exists」と
「Manage toggle states: aria-expanded / aria-pressed」を推奨する。

## 決定

- `.reader-content` から `aria-label="Chapter content"` を削除し、
  `aria-labelledby="reader-chapter-title"` に一本化する。
  - ADR-0058 の「aria-label 維持」決定は本 ADR で **supersede** する。
- `#reader-theme` に `aria-pressed` を追加する。
  - バインドは `dark`（手動設定）ではなく **`effective_dark`（実効値）**。
  - 可視ラベル（"Light" / "Dark"）も `effective_dark` へ合わせる。
  - `title` / `aria-label` / `aria-keyshortcuts="d"` / `on:click` は維持。
- `#reader-theme-auto` は `follow_system` バインドを維持する。
- E2E に 1 件追加し、期待総数 250 → **251**（check 234 + equal 17）。
  - aria-label が null／空、aria-labelledby が `reader-chapter-title`。
  - `#reader-theme` の aria-pressed が `<html>` の dark class
    （= effective_dark）と一致。
  - 既存 check「本文領域に role・aria-label・tabindex が付く」は
    aria-label アサーションを外し「role・tabindex」へ改名（件数不変）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| aria-labelledby に統一＋実効値告知（採用） | 冗長名と状態の食い違いを防ぐ |
| aria-label 維持 | 可視ラベルがある場合は非推奨 |
| `dark`（手動値）を aria-pressed に使用 | OS 追従時に表示と状態が食い違う |

## 影響

- `follow_system=true` のとき、ボタンの可視ラベルと aria-pressed は
  従来の手動 `dark` 基準から実効値基準へ変わる。
- `dark_action` 自体は従来どおり手動 `dark` をトグルする実装のまま。
- 章未選択時は `current_title` が空文字のため
  accessible name も空になる（未決定）。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- E2E「Reader の ARIA 属性が可視ラベルとトグル状態に整合する」が ok。
- AST 監査: check 234 + equal 17 = 251、check 名一意 251・重複 0（ADR-0182）。
- `python3 -m py_compile`、`cargo fmt --all -- --check`、wasm check、
  `./scripts/verify.sh`: すべて成功。
- E2E **251 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 章未選択時の accessible name フォールバック。
- computed accessible name の実機検証（Chrome 157 のみ）。
- `aria-pressed` と可視ラベルの意味付けを「操作対象」と「現在状態」
  のどちらに統一するか（現状は両方実効値）。
