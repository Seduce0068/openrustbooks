# Goal

環境変数 `OPENRUSTBOOKS_AI_MAX_RETRIES` の設定マッピングを検証・修正する（ADR-0120 / 0121 の未決定）。

## Scope

### 背景

`AppConfig::load()` は `Env::prefixed("OPENRUSTBOOKS_").split("_")` を使う。
`split("_")` は全ての `_` を階層区切り `.` に変換すると推測され、
`OPENRUSTBOOKS_AI_MAX_RETRIES` が `ai.max.retries` になり、
serde フィールド `ai.max_retries` に届かない可能性がある。

### 成果物

- `core/src/config.rs`: 変更（必要ならマッピング修正）
- `docs/decisions/0122-env-var-mapping.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. figment 0.10.19 の `split("_")` の実際の挙動をローカルクレートで確認する。
2. `OPENRUSTBOOKS_AI_MAX_RETRIES` が `AiConfig.max_retries` に届くかテストで確認する。
3. 届かない場合は、既存の環境変数を壊さずに修正する。
4. 既存の 200 テスト（native 200 + wasm 3）、E2E 228 項目を壊さない。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- 新規依存を追加しない。
- `master` へ push する。force push はしない。
