# Goal

読書位置の全消去（Clear data）に確認ダイアログを追加する（ADR-0151 の高優先度 UX 残タスク）。

## Scope

### 背景

`on_clear_data()` は確認なしで `clear_reading_positions()` を呼ぶ。
履歴の全消去はネイティブ dialog で確認を取るのに、読書位置は取り消し不能。
ADR-0018 / final-report C03 で残存が指摘されている。

### 成果物

- `web/src/lib.rs`: 変更（`window.confirm()` による確認）
- `tools/verify-web-reader.py`: 変更（E2E 1 件追加、244 → 245）
- `docs/decisions/0171-clear-data-confirmation.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. Clear data クリック時に `window.confirm()` で確認を求める。
2. キャンセル時は何も削除せず、status も変更しない。
3. 既存の E2E「Clear data で削除される」は confirm を true でオーバーライドして通す。
4. 新規 E2E で confirm=false のとき位置が残ることを検証。
5. 期待総数 244 → 245。

## Acceptance criteria

- E2E 245 項目が成功する。
- 6 ゲート + audit + wasm-test が成功する。

## Constraints

- 新規依存を追加しない。
- 既存の削除文言・件数表示は維持する。
- `master` へ push する。force push はしない。
