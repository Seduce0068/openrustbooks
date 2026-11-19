# Goal

AppConfig::save() を tmp + rename の原子書き込みへ変更する（R6 TD-L09、ADR-0209）。

## Scope

### 成果物

- `core/src/config.rs`: save_to_dir / atomic_write / 0600 保護、unit 7 件
- `docs/decisions/0209-atomic-config-save.md`: 追加
- `todo.md`: TD-L09 を完了へ移動

### 決定

- tmp 名 `<file>.<pid>.<counter>.tmp`、create_new + 最大 128 回。
- Unix は mode(0o600) + set_permissions で 0600 維持。
- write/rename 失敗で tmp 削除、元設定は rename 成功時のみ置換。
- lost update（TD-L10）は未解消として ADR に記録。

## Acceptance criteria

- core 261 tests / E2E 251、全ゲート成功（検証済み）。
