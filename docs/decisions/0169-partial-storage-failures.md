---
type: Decision
title: "ADR-0169: 部分ストレージ失敗の可視化"
description: "読書位置の全消去・インポートの部分失敗を操作単位で通知する決定を記録する。"
tags:
  - ADR
  - web
  - data
generated:
  by: dsh
  at: "2026-11-19T00:20:00+09:00"
status: accepted
---

## 背景

ADR-0154 で localStorage 保存失敗の通知を実装したが、
`clear_reading_positions()` は `remove_item()` の失敗をスキップし、
`import_positions()` も `set_item()` の失敗を件数に数えないだけだった
（ADR-0154 の未決定）。

## 決定

- `clear_reading_positions()` は `(removed, failed)` を返し、
  ループ後、1 件でも失敗した場合のみ
  `report_save_failure("全消去", SaveErrorTarget::Reader)` を **1 回**発行。
- `import_positions()` は位置／履歴の `set_item()` 失敗を両ループ後にまとめ、
  `report_save_failure("インポート", SaveErrorTarget::Reader)` を 1 回発行。
  - 既存の `Result<(usize, usize), String>` 戻り値は維持。
  - 失敗フラグは `&mut bool` で呼び出し側へ渡す。
- 呼び出し側 `on_clear_data()` / `on_import_change()` は、
  失敗時に対向の成功 `set_status` を実行しない
  （`dispatch_event` が同期のため、成功文言で失敗通知が上書きされるのを防ぐ）。
- 通知内容は固定 context（「全消去」「インポート」）のみ。
  キー名・値・件数・生エラー・E2E エラー文言は載せない。
- E2E に 1 件追加し、期待総数 242 → **243**（check 226 + equal 17）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 操作単位で 1 回通知（採用） | aria-live ノイズを抑えつつ失敗を通知 |
| 失敗ごとに通知 | 同一操作で N 回読み上げられる |
| 成功数だけ返す | 部分失敗が利用者に見えない |

## 影響

- 成功文言（"Removed ..." / "Imported ..."）は失敗時に出なくなる。
- `report_save_failure()` の context は固定 2 値が増えただけ。
- `SaveErrorTarget::Reader` の送信先分離（ADR-0165）は維持。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- E2E「全消去の保存失敗時に status と console へ通知する」が ok:
  - status に固定文言、成功文言が出ない（`"Removed" not in status`）。
  - console.error に「全消去 の保存に失敗しました」が 1 回のみ。
  - 2 件失敗でも通知は 1 回。
- E2E **243 項目すべて成功**。
- 全単体テスト core 202 + desktop 56 + web 2、wasm-test 10、
  audit 脆弱性 0 件。
- `python3 -m py_compile`、`cargo fmt --all -- --check`、wasm check、
  `./scripts/verify.sh`: すべて成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- import の設定復元（`save_settings`）と位置／履歴通知の二重化
  （「表示設定」と「インポート」を統合するか）。
- 部分失敗時の件数表示（成功件数のみ／失敗件数も併記）。
- 通知の寿命（再成功で消える／自動消滅／手動 dismiss、ADR-0154 から継続）。
- prune 失敗の格上げ、`QuotaExceededError` の分類（ADR-0154 から継続）。
