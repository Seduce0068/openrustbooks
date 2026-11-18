# Goal

秘密情報だけを削除する dedicated command を追加する（ADR-0131 / 0132 の未決定）。

## Scope

### 背景

`has_api_key` 等は読み取り専用で、`ConfigPatch` で受け取っても無視される。
そのため API キー／クライアントシークレット／リフレッシュトークンを
個別に削除する手段が無い（`reset_config` は全設定を既定に戻す）。

### 成果物

- `desktop/src/commands/config.rs`: 変更（`clear_secrets` command とテスト）
- `desktop/src/main.rs`: 変更（command 登録）
- `docs/decisions/0134-secret-removal-command.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `clear_secrets(state, targets: SecretTargets)` を追加し、指定された
   秘密フィールドだけを `None` / 空文字にして保存する。
2. `SecretTargets` は `api_key` / `client_secret` / `refresh_token` の
   bool フィールドを持つ（既定 false＝消さない、fail-closed）。
3. 既存 `merge_secrets` が `None` / 空を「未変更」とみなして復元するため、
   この command は merge を通さずに直接 state を更新する。
4. テストを追加する（指定したものだけ消えること、他設定は不変）。
5. 既存の 234 テスト + wasm 3、E2E 229 項目を壊さない。

## Acceptance criteria

- 6 ゲート + audit + wasm-test + E2E が成功する。

## Constraints

- 新規依存を追加しない。
- Web UI への接続は今回行わない（command の準備のみ）。
- `master` へ push する。force push はしない。
