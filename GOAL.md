# Goal

エディタ下書きをデバウンス自動保存し、未保存状態を可視化する（R6 TD-H05、ADR-0179）。

## Scope

### 成果物

- `web/src/lib.rs`: 800ms デバウンス自動保存、未保存表示、タイマー cleanup
- `tools/verify-web-reader.py`: E2E 1 件、247 → 248
- `docs/decisions/0179-editor-autosave.md`: 追加（beforeunload 見送りを記録）
- `todo.md`: TD-H05 を完了へ移動

### 決定

- 入力直後に未保存表示、800ms 後に自動保存。
- 同一文言の再 set をスキップし aria-live の発話過多を防止。
- beforeunload は CDP のダイアログブロッキングを避けるため実装しない。

## Acceptance criteria

- E2E 248 項目、全ゲート成功（検証済み）。
