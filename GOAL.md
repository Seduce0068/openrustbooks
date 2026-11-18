# Goal

読書位置の全消去（Clear data）に `window.confirm()` の確認ダイアログを追加する（ADR-0171）。

## Scope

### 背景

`on_clear_data()` は確認なしで `clear_reading_positions()` を呼んでいた。
ADR-0040 は履歴側で `window.confirm()` を却下したが、
本タスクは Reader の Clear data に限り方針を転換する。

### 成果物

- `docs/decisions/0171-clear-data-confirmation.md`: 追加
- `GOAL.md`: 最終契約へ更新

### 決定

- Clear data クリック時に `window.confirm_with_message()` で確認する。
- `Ok(false)` なら削除・status 変更・保存失敗通知のいずれも行わない。
- window 無し／confirm 例外時は既存どおり削除する fail-safe。
- 既存の削除文言・設定残留・件数表示は維持。
- E2E は `window.confirm` をオーバーライドして実ダイアログを出さず、
  期待総数を 244 → 245 に更新。

## Acceptance criteria

- E2E 245 項目、全ゲート成功（検証済み）。
