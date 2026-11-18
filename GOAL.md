# Goal

設定保存と AI サービス再初期化を 1 トランザクション化する（R6 PR-H04）。

## Scope

### 背景

`update_config` / `update_config_patch` / `update_ai_config` / `update_lint_config` /
`clear_secrets` は `state.update_config()` → `state.save_config()` →
`state.refresh_ai_service()` の順でコミットする。`refresh_ai_service()` が失敗すると
メモリと config.toml には新設定が残る一方 AI service だけ None になり、
利用者には失敗が返るが再起動後は新設定が有効という中途半端な状態になる。

### 成果物

- `desktop/src/state/mod.rs`: 変更（候補設定で service を準備し、保存成功後にコミット）
- `desktop/src/commands/config.rs`: 変更（全更新コマンドを新 API へ統一）
- `docs/decisions/0177-config-commit-transaction.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. AI service の生成を候補 `AppConfig` に対して先に行い、失敗時はメモリ・ディスク・service を一切変更しない。
2. ディスク保存が失敗した場合はメモリ設定を旧値へロールバックし、旧 service を維持する。
3. 保存成功後にのみ新設定と新 service をコミットする（無効化なら service は None）。
4. 既存の validation / merge_secrets / no-op 判定を維持する。
5. unit test を追加（service 生成失敗時に設定不変、成功時に service 更新）。
6. 全ゲート成功（E2E 247 維持）。

## Acceptance criteria

- 全ゲート成功、E2E 247 項目成功。

## Constraints

- 新規依存を追加しない。
- 既存の fail-closed（service 生成失敗時 None）を維持する。
- `master` へ push する。force push はしない。
