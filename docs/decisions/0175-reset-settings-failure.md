---
type: Decision
title: "ADR-0175: 設定リセット時の保存失敗表示"
description: "設定リセットが保存失敗通知を成功文言で上書きしないようにする決定を記録する。"
tags:
  - ADR
  - web
  - ux
generated:
  by: dsh
  at: "2026-11-19T02:25:00+09:00"
status: accepted
---

## 背景

`on_reset_settings()` は `save_settings()` の成否を検査せず、
直後に `"Display settings reset."` を `set_status()` していた。
`report_save_failure()` は `window.dispatch_event()` で同期に
Reader リスナーを走らせるため、成功文言が失敗通知を上書きしていた
（R6: PR-H05）。

## 決定

- `save_settings()` を `-> bool` に変更する。
  - `window` / `local_storage()` / `serde_json::to_string()` / `set_item()`
    の全失敗経路で `report_save_failure("表示設定", SaveErrorTarget::Reader)`
    を呼び `false` を返す。
  - 成功時のみ `true`。
- `on_reset_settings()` は `true` のときだけ
  `"Display settings reset."` を表示する。
- 既存の他呼び出し（import / persist）は戻り値を無視するが、
  内部通知を維持するため挙動不変。
- E2E に 1 件追加し、期待総数 246 → **247**（check 230 + equal 17）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| bool 返却＋呼び出し側ガード（採用） | 通知を一箇所に集約しつつ成功文言を制御できる |
| 呼び出し側で localStorage を再検証 | 重複し、redaction も難しい |
| 成功文言を廃止 | UX が低下する |

## 影響

- 失敗時: メモリ上の signal は既定値へ戻るが、localStorage は旧値のまま
  （E2E では `finally` でキーを復元）。
- `serde_json::to_string()` 失敗が新たに通知対象になった
  （実質発生しないが挙動変更）。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- E2E「リセット保存失敗時に成功文言を出さず status と console へ通知する」が ok。
- E2E **247 項目すべて成功**。
- 全単体テスト core 210 + desktop 56 + web 2、wasm-test 10、
  audit 脆弱性 0 件。
- `python3 -m py_compile`、`cargo fmt --all -- --check`、wasm check、
  `./scripts/verify.sh`: すべて成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- import 経路の設定保存失敗を `save_failed` へ統合するか
  （ADR-0169 の未決定を継続）。
- 既存 E2E の一部 `setItem` オーバーライドに `try/finally` が無い箇所の修正。
- 通知の寿命（再成功で消える／自動消滅／手動 dismiss）。
- `QuotaExceededError` の分類（ADR-0154 から継続）。
