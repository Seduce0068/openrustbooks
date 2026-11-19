# Goal

figment の環境変数マッピングを手動上書きから明示 map 方式へ置き換える（R6 TD-M08b、ADR-0122 の未決定）。

## Scope

### 背景

`AppConfig::load()` は `Env::prefixed("OPENRUSTBOOKS_").split("_")` を使うが、
figment の split 挙動で `OPENRUSTBOOKS_AI__MAX_RETRIES` のような
アンダースコア包含フィールドが正しく届かず、
`apply_env_max_retries()` で `OPENRUSTBOOKS_AI_MAX_RETRIES` を手動読み込みしている。
新しい env 対応フィールドが増えるたびに手動マッピングが増える。

### 成果物

- `core/src/config.rs`: 変更（明示 map／プロバイダ、手動上書きの整理）
- `docs/decisions/0204-env-mapping.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. figment の env マッピングを、既存の `OPENRUSTBOOKS_AI_MAX_RETRIES` を含む
   明示的な変換へ置き換える。
2. 既存の環境変数名・TOML / IPC 優先順位・不正値時の既定維持を壊さない。
3. `apply_env_max_retries()` は不要なら削除し、必要なら役割を限定する（判断を記録）。
4. env マッピングの unit test を追加／維持する（既存 3 件を含む）。
5. 全ゲート成功（E2E 251 維持）。

## Acceptance criteria

- 全ゲート成功、E2E 251 項目成功。

## Constraints

- 環境変数名の互換を壊さない（利用者設定を突然無効化しない）。
- 新規依存を追加しない。
- `master` へ push する。force push はしない。
