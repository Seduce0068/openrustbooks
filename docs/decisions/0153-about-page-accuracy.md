---
type: Decision
title: "ADR-0153: About 画面の表示内容の正確化"
description: "About 画面のライセンスと対応形式を実装事実に合わせる決定を記録する。"
tags:
  - ADR
  - web
  - docs
generated:
  by: dsh
  at: "2026-11-18T19:55:00+09:00"
status: accepted
---

## 背景

ADR-0151 の探索で、About 画面が
`License: MIT`、`Supports: EPUB, PDF, Markdown, TXT, ZIP` と表示していたことが判明した。
実際は `Cargo.toml:12` = `AGPL-3.0-or-later`、
`README.md:81` は EPUB 閲覧のみ実装済み、他形式は設計段階としている。

## 決定

- About 画面（`web/src/lib.rs:172-176`）を次に変更:
  - `EPUB reading: available`
  - `TXT / Markdown / PDF / ZIP: planned`
  - `License: AGPL-3.0-or-later`
- 旧 MIT 本文は `THIRD_PARTY_NOTICES.md` に保持し、ライセンス本文自体は変更しない。
- E2E に 1 件追加（`/about` へ遷移し、`.about-page` の表示と
  3 つの正しい文字列、旧誤表示が無いことを検証）。
- 期待総数を 231 → **232**（`check 215 + equal 17`）に更新。
- E2E チェックは末尾に配置する（`Page.navigate` によるフルリロードで
  外部リンクのオーバーライド等が初期化されるため。後続チェック追加時は
  再セットアップが必要）。
- `.about-page` は CSS を持たないテストフック契約として維持する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 実装事実に合わせて文言修正（採用） | 利用者への誤案内を解消 |
| 古い MIT / 全形式対応のまま | ライセンスと実装状況の両方で不正確 |
| README を About に合わせる | 実装が追いついていないため不適切 |

## 影響

- 表示文言のみの変更、機能には影響なし。
- 新規依存なし。

## 検証（Runtime Proof）

- `python3 -m py_compile tools/verify-web-reader.py`、`cargo fmt --all -- --check`、
  wasm32 web check: すべて成功。
- `./scripts/verify.sh` rc=0、E2E **232 項目すべて成功**
  （新規チェック「About 画面に正しいライセンスと対応形式が表示される」も ok）。
- 全単体テスト core 196 + desktop 56 + web 2、wasm-test 4、
  audit 脆弱性 0 件。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `README.md:94` の「設計段階」に EPUB も列挙されている既存の不整合
  （本次は表示文言のみの変更のため対象外）。
- About 文言の日本語化 / i18n（ADR-0151 から継続）。
- E2E を SPA 遷移へ変更して状態保持を狙うか（現状は末尾配置で回避）。
- 旧文字列の否定判定を完全一致から部分一致へ強化するか。
