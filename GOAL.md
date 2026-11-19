# Goal

figment の環境変数マッピングを明示化する（R6 TD-M08b、ADR-0204）。

## Scope

### 成果物

- `core/src/config.rs`: map_env_key / include_env_key / Env filter、unit 7 件
- `docs/decisions/0204-env-mapping.md`: 追加（ADR-0122 の未決定を解消）
- `todo.md`: TD-M08b を完了へ移動

### 決定

- OPENRUSTBOOKS_AI_MAX_RETRIES（アンダースコア 1 つ）を ai.max_retries へ明示マップ。
- 不正値は生値検証 filter で除外し、toml/default を維持（起動失敗を防ぐ）。
- apply_env_max_retries は extract 後の防御的フォールバックとして維持。

## Acceptance criteria

- core 255 tests / E2E 251、全ゲート成功（検証済み）。
