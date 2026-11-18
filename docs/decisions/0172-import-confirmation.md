---
type: Decision
title: "ADR-0172: インポートの確認ダイアログ"
description: "バックアップの取り込み前に window.confirm() の確認を求める決定を記録する。"
tags:
  - ADR
  - web
  - data
generated:
  by: dsh
  at: "2026-11-19T01:20:00+09:00"
status: accepted
---

## 背景

ADR-0157 で現在の設定／履歴はインポートから保護されたが、
読書位置は ADR-0032 の `savedAt` マージにより上書きされ得る。
Clear data には確認を追加済み（ADR-0171）だが、import は未確認だった。

## 決定

- `on_import_change()` でファイル取得後・読み込み前に
  `window.confirm_with_message("バックアップを取り込みます。現在の読書位置を上書きする場合があります。よろしいですか？")` を呼ぶ。
- `Ok(false)` なら `input.set_value("")` して return。
  読み込み・取り込み・status 変更を行わない。
- `window` 無し／confirm 例外時は fail-safe で続行する（既存どおり）。
- 成功時も `input.set_value("")` し、同じファイルを再選択可能にする。
- E2E では `select_import_file()` ヘルパーで
  `window.confirm = () => true` を適用し、`try/finally` で復元する
  （既存 import 11 箇所すべてを置換）。CDP の実ダイアログは出さない
  （ADR-0171 の方針を踏襲）。
- 新規 E2E で confirm=false のとき番兵キーと status が変わらないことを検証。
- 期待総数 245 → **246**（check 229 + equal 17）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| `window.confirm()`（採用） | Clear data と同方針、新規依存なし |
| 確認なし | 新しいバックアップで現在位置が上書きされ得る |
| `<dialog>` に統一 | ADR-0040 / 0171 の履歴側実装を変更する範囲外 |

## 影響

- ADR-0171 の未決定「import 側の確認保護」を解消。
- 既存 import E2E 11 箇所に confirm オーバーライドを追加したが、
  既存の検証内容・文言・件数は不変。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- E2E「confirm=false ではインポートせず位置キーと status が変わらない」が ok。
- E2E **246 項目すべて成功**。
- 全単体テスト core 202 + desktop 56 + web 2、wasm-test 10、
  audit 脆弱性 0 件。
- `python3 -m py_compile`、`cargo fmt --all -- --check`、wasm check、
  `./scripts/verify.sh`: すべて成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 確認文言の i18n（ADR-0151 / 0171 から継続）。
- `<dialog>` との UI/UX 統一。
- 5 ブラウザ実機での confirm 挙動（現状は Chrome 157 headless で検証）。
