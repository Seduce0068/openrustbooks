# Goal

EditorConfig / ReaderConfig の死設定を正直化する（R6 TD-M19）。

## Scope

### 背景

`AppConfig.editor` / `.reader` の値は ConfigView 経由で設定画面に表示されるが、
Editor / Reader の実動作には接続されていない。
実動作の権威は localStorage の `ReaderSettings`（ADR-0211）と
エディタ下書き（ADR-0179）であり、ADR-0216 の streaming と同型の誤認リスクがある。

### 成果物

- `web/src/lib.rs`: 変更（Editor / Reader セクションへ「未接続」注記）
- `docs/decisions/0217-editor-reader-dead-config.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映
- `docs/index.md` / `docs/decisions/index.md` / `README.md` / `docs/log.md`: 更新

### 必須事項

1. Editor セクションへ「config.toml の設定値であり、エディタ動作には未接続」の注記を追加する。
2. Reader セクションへ「config.toml の設定値であり、読書表示は localStorage 設定が権威（ADR-0211）」の注記を追加する。
3. `ConfigView` / `ConfigPatch` / `SettingsEditorView` / `SettingsReaderView` のフィールドは削除しない。
4. E2E 253 を維持する（文言依存なしを確認）。
5. 全ゲート成功。

## Acceptance criteria

- 全ゲート成功、E2E 253 項目成功。

## Constraints

- 設定ファイル・DTO の互換を壊さない。
- フィールド削除は未決定として ADR に残す。
- `master` へ push する。force push はしない。
