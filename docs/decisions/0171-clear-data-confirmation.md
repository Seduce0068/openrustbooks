---
type: Decision
title: "ADR-0171: Clear data の確認ダイアログ"
description: "読書位置の全消去に window.confirm() の確認を追加する決定を記録する。"
tags:
  - ADR
  - web
  - ux
generated:
  by: dsh
  at: "2026-11-19T01:00:00+09:00"
status: accepted
---

## 背景

ADR-0151 の探索で、履歴の全消去はネイティブ dialog で確認を取る一方、
Reader の Clear data（読書位置の全消去）は確認なしで
`clear_reading_positions()` を呼ぶことが指摘された（ADR-0018 / final-report C03）。
取り消し不能な操作である。

ADR-0040:49 は `window.confirm()` を「スタイル制御ができず、E2E も扱いにくい」
として却下し、履歴側で `<dialog>` を採用した。
本 ADR は **Reader の Clear data に限り**方針を転換する。

## 決定

- `on_clear_data()` の先頭で
  `window.confirm_with_message("すべての読書位置を削除します。よろしいですか？")` を呼ぶ。
- `Ok(false)` の場合は early return し、
  削除・status 変更・保存失敗通知（ADR-0169）のいずれも発行しない。
- `window` が無い／confirm が例外を投げる場合は `None` として
  既存どおり削除する（fail-safe）。
- 既存の削除文言 `Removed {removed} saved reading positions.`、
  設定が残る挙動、件数表示は維持する。
- 履歴側の `<dialog>`（ADR-0040）は据え置き、統一はしない。
- E2E では実ダイアログを出さず、JS 側で `window.confirm` を
  `() => true` / `() => false` へ差し替える。
  3 箇所すべて `try/finally` で復元する（CDP の 10 秒タイムアウト回避）。
- E2E に 1 件追加し、期待総数 244 → **245**（check 228 + equal 17）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| `window.confirm()`（採用） | 新規依存なしで確実に確認できる。E2E はオーバーライドで回避 |
| `<dialog>` に統一 | ADR-0040 の実装を変更する必要があり範囲外 |
| 確認なし | 取り消し不能な削除が誤操作で起きる |

## 影響

- ADR-0040 の `window.confirm()` 却下は「履歴側」に限定される。
  本 ADR は Reader の Clear data に対する方針転換である。
- CDP は `Page.javascriptDialogOpening` を診断対象に含まず、
  実 confirm が出ると `Runtime.evaluate` が 10 秒で fail-closed になる
  （ADR-0144）。E2E オーバーライドが必須。
- 新規 E2E「confirm=false では削除も status 変更も保存失敗通知も出ない」が
  キャンセル経路を検証。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- E2E **245 項目すべて成功**。
  - 新規チェック: `confirm=false では削除も status 変更も保存失敗通知も出ない`。
  - 既存の「Clear data で削除される」「設定は残る」「Removed 文言」も維持。
- 全単体テスト core 202 + desktop 56 + web 2、wasm-test 10、
  audit 脆弱性 0 件。
- `python3 -m py_compile`、`cargo fmt --all -- --check`、wasm check、
  `./scripts/verify.sh`: すべて成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 確認文言の i18n（ADR-0151 の i18n 未決定を継続）。
- `<dialog>` との UI/UX 統一。
- 5 ブラウザ実機での confirm 挙動（現状は Chrome 157 headless で検証）。
- ~~import 側の確認保護~~ ADR-0172 で confirm 追加済み。
