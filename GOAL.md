# Goal

読書位置の全消去・インポートの部分失敗を可視化する（ADR-0154 の未決定）。

## Scope

### 背景

`clear_reading_positions()` は `remove_item()` の失敗を黙ってスキップし、
削除成功数のみ返す。`import_positions()` も `set_item()` の失敗を
件数に数えないだけで、利用者へ通知しない（ADR-0154 の未決定）。

### 成果物

- `web/src/lib.rs`: 変更（部分失敗の通知）
- `tools/verify-web-reader.py`: 変更（E2E 1 件、242 → 243）
- `docs/decisions/0169-partial-storage-failures.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. 全消去で 1 件でも `remove_item()` が失敗した場合、
   `report_save_failure("全消去", SaveErrorTarget::Reader)` を発行する。
2. インポートで位置／履歴の `set_item()` が失敗した場合も
   `report_save_failure("インポート", SaveErrorTarget::Reader)` を発行する。
3. 成功時は既存の status 文言を維持する。
4. 通知にキー名・値・サイズは載せない（固定 context のみ）。
5. E2E に 1 件追加し、期待総数 242 → 243。
6. 既存 242 項目と全ゲートを壊さない。

## Acceptance criteria

- E2E 243 項目が成功する。
- 6 ゲート + audit + wasm-test が成功する。

## Constraints

- 新規依存を追加しない。
- 既存の成功文言・件数表示は維持する。
- `master` へ push する。force push はしない。
