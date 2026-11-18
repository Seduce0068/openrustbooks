# Goal

設定リセットが保存失敗通知を成功文言で上書きする問題を修正する（R6 PR-H05、ADR-0175）。

## Scope

### 成果物

- `web/src/lib.rs`: save_settings の bool 化と reset 側のガード
- `tools/verify-web-reader.py`: E2E 1 件、246 → 247
- `docs/decisions/0175-reset-settings-failure.md`: 追加
- `todo.md`: PR-H05 を完了へ移動

### 決定

- 全失敗経路で report_save_failure を呼び false を返す。
- 成功時のみ "Display settings reset." を表示。
- E2E は try/finally で setItem / console.error / 設定キーを復元。

## Acceptance criteria

- E2E 247 項目、全ゲート成功（検証済み）。
