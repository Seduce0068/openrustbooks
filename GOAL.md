# Goal

`extra_headers` を IPC のマスク対象に含める（ADR-0098 の未決定、baseurl-auditor の指摘）。

## Scope

### 背景

`OpenAiConfig.extra_headers` は DTO（`OpenAiConfigView`）で JS へそのまま公開され、
`update_config` / `update_ai_config` からも任意の値が設定できる。
利用者が `Authorization` 等を入れると平文で config.toml と IPC に露出する。

### 成果物

- `desktop/src/commands/config.rs`: 変更（DTO から extra_headers を除外、マージ維持）
- `core/src/config.rs`: 変更（Debug マスクに extra_headers を含める）
- `docs/decisions/0102-extra-headers-redaction.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `OpenAiConfigView` から `extra_headers` フィールドを削除し、JSON に含まれないようにする。
2. `merge_secrets` は `extra_headers` も保持中の実値をマージし、更新時に消えないようにする（値は IPC で送らない）。
3. `OpenAiConfig` の Debug で `extra_headers` の値を `"***"` にマスクする。
4. テストを追加する（DTO JSON に extra_headers が無いこと、マージで保持されること、Debug マスク）。
5. 既存の 156 テスト、E2E 228 項目を壊さない。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- 新規依存を追加しない。
- UI の配線は変更しない（未接続のまま）。
- `master` へ push する。force push はしない。
