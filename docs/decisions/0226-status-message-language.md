---
type: Decision
title: "ADR-0226: status 文言の日本語統一"
description: "lang=\"ja\" 固定の Web UI で Reader status を日本語へ統一し、aria-live 分離は後段へ残す決定を記録する。"
tags:
  - ADR
  - web
  - accessibility
generated:
  by: dsh
  at: "2026-11-19T16:14:00+09:00"
status: accepted
---

## 背景

`web/index.html` は `<html lang="ja">` 固定（ADR-0057）だったが、
Reader の status には英語文言が残り、Editor / Settings の日本語と混在していた
（R6 TD-M11）。
`aria-live` のerror / info 分離は ADR-0021 で明示的に却下済みで、
DOM 契約変更を伴うため第二段の別 ADR に委ねる。

## 決定

- Reader status の英語文言 22 パターン／24 箇所を日本語へ置換する。
  動的値（件数、ページ番号、ファイル名、バージョン、寸法、MiB 上限）は書式を維持する。
- ADR-0212 の redaction 契約を継続する。
  固定エラー文言のみで、生エラー／パス／本文／型名は status へ出さない。
- `report_reader_error(operation)` の console 操作名（英語）は診断用として維持する
  （E2E の既存契約）。
- `role="status" aria-live="polite"` の属性と要素構造は変更しない。
- E2E の文言期待 23 箇所を同期更新する。
  動的値は正規表現で書式検証し、check 総数 254 を維持する。
- UI ラベル／ボタン／aria-label／ヘルプ本文の英語は status ではないため本次対象外。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 文言のみ日本語統一（採用） | DOM / a11y 契約を壊さず lang と一致する |
| error / info を別 aria-live へ分離 | ADR-0021 で却下済み、第二段の別 ADR が必要 |
| 日英許容の曖昧な検証 | 文言変更を検出できなくなるため採用しない |

## 影響

- status が日本語話者の発音規則で読み上げられる（`lang="ja"` と整合）。
- E2E の完全一致文言（EPUB open エラー）も日本語へ更新。
- 原文を固定文言として記録した ADR-0198 / 0212 / 0213 は
  作成時点の実績として保持し、本 ADR で上書きする。
- 新規依存なし、unsafe なし、E2E 254 維持。

## 検証（Runtime Proof）

- `python3 -m py_compile tools/verify-web-reader.py`、
  `cargo fmt --all -- --check`、
  `cargo check -p openrustbooks-web --target wasm32-unknown-unknown --locked`: 成功。
- AST 抽出: check 237 + equal 17 = **254**（`Checker(expected=254)` を維持）。
- `./scripts/verify.sh`: 全ゲート成功、E2E **254 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- error / info / warning の分類表
  （Page not found、大画像、remote resources、未保存表示の扱い）。
- `aria-live="assertive"` の別領域を設ける場合の配置と二重読み上げ対策
  （ADR-0021 / 0154 / 0212 を継続）。
- i18n クレート導入の是非（今回は追加せず `lang="ja"` 固定）。
- 通知寿命（自動消去・保持時間、ADR-0154 を継続）。
