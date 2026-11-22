# Goal

backup_failed_config の権限失敗時に退避をロールバックする（第2cycle R6 / R2-RC01）。

## Scope

### 背景

rename 成功後に set_permissions(0o600) が失敗すると、
機微情報を含む config.toml が 0644 のまま `*.failed-*` に残る。
呼び出し側は「退避失敗」と表示するが、実際は退避済みで状態が食い違う。

### 成果物

- `core/src/config.rs`: 変更
- `docs/decisions/0320-config-backup-rollback.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. Unix で set_permissions 失敗時に rename を元へ戻す。
2. ロールバック失敗时は固定文言の CoreError で fail-closed を維持する。
3. 既存の retention test / 0600 test / 連続起動 test を壊さない。
4. 権限失敗を模した単体 test を追加する（可能なら cfg(unix) の mod bit 操作）。
5. verify.sh 全ゲート成功、E2E 256 件・WDIO 維持。

## Constraints

- unsafe FFI / 新規依存は使わない。
- 絶対パス / 設定値をエラーへ出さない。
- `master` へ push する。force push はしない。

## 完了記録

- set_permissions 失敗時に退避を元へ戻し、元へ 0600 再適用。
- ロールバック失敗時は退避先を残して原文保持を優先。
- 模擬 test 3 件追加、core 355 tests、verify.sh 全ゲート成功（E2E 256、WDIO 1）。
- 退避先権限露出 / 実 chmod test は ADR-0320 の未決定。
