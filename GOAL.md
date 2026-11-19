# Goal

未検出ストレージバックエンド / AI provider の unit test を追加する（R6 R4-unit#3 残）。

## Scope

### 背景

設定・保存・AI 主要モジュールのテストは core 267 件まで拡充済みだが、
`StorageManager::set_default_backend()` の未検出バックエンド経路
（`core/src/io.rs:232`）と `create_client()` の未知 provider 経路
（`core/src/ai_markdown/client.rs:159`）に専用 test が無い。

### 成果物

- `core/src/io.rs`: 変更（未検出 backend の unit test）
- `core/src/ai_markdown/client.rs`: 変更（未知 provider の unit test）
- `docs/decisions/0231-unknown-backend-tests.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. `set_default_backend("missing")` が `CoreError::Storage("Backend not found: missing")` を返すことを検証する。
2. 既存 backend を登録した場合の成功経路も維持されることを検証する。
3. `create_client()` で未知 provider が `CoreError::AiProvider` を返すことを検証する。
4. エラー文言にパス・秘密・本文が含まれないことを確認する。
5. 全ゲート成功（E2E 254 維持）。

## Acceptance criteria

- core test が 2 件以上増加し、全ゲート成功。

## Constraints

- 本番コードの挙動は変更しない（test 追加のみ）。
- 新規依存を追加しない。
- `master` へ push する。force push はしない。
