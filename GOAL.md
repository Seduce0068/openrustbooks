# Goal

atomic write の一時ファイル名を一意化する（R6 PR-M06、ADR-0188）。

## Scope

### 成果物

- `core/src/io.rs`: ATOMIC_WRITE_ID / atomic_temp_path / create_atomic_temp / write_atomic、unit 4 件
- `docs/decisions/0188-atomic-tmp-uniqueness.md`: 追加
- `todo.md`: PR-M06 を完了へ移動

### 決定

- tmp 名 `<file>.<pid>.<counter>.tmp`、create_new + 128 回リトライ。
- 失敗時 best-effort 削除、overwrite / create_parents 契約維持。
- E2E 249 維持。

## Acceptance criteria

- core 228 / E2E 249、全ゲート成功（検証済み）。
