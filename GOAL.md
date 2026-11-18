# Goal

設定保存用の patch DTO と `update_config_patch` command を追加する（ADR-0128 の未決定）。

## Scope

### 背景

`get_config` は `ConfigView`（secret / パス除外 DTO）を返すが、
`update_config` は完全な `AppConfig` を要求する。
Web 側が DTO をそのまま送るとデシリアライズ失敗し、
完全な AppConfig を再構成すると隠しフィールドが既定値で上書きされる。

### 成果物

- `desktop/src/commands/config.rs`: 変更（`ConfigPatch` と `update_config_patch` を追加）
- `desktop/src/main.rs`: 変更（command 登録）
- `docs/decisions/0131-config-patch-command.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `ConfigView` と同じ公開フィールドを持つ `ConfigPatch`（Deserialize）を追加する
   （secret は `has_*` ではなく実値を Option で受け取るか、今回は受け取らない）。
2. `update_config_patch(state, patch: ConfigPatch)` を追加し、
   現在の `AppConfig` に patch の公開フィールドだけを上書きしてから
   既存の `merge_secrets` / `validate_ai_base_urls` / 差分検知 /
   `refresh_ai_service()` と同じ処理を行う。
3. 旧 `update_config` は互換性のため維持する。
4. テストを追加する（patch が秘密 / パスフィールドを保持すること）。
5. 既存の 227 テスト + wasm 3、E2E 229 項目を壊さない。

## Acceptance criteria

- 6 ゲート + audit + wasm-test + E2E が成功する。

## Constraints

- 新規依存を追加しない。
- Web UI の接続は今回行わない（command の準備のみ）。
- `master` へ push する。force push はしない。
