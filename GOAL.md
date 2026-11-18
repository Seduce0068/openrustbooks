# Goal

秘密削除時に削除対象名だけをログに残す（ADR-0134 の未決定）。

## Scope

### 背景

`clear_secrets` は削除対象の値を一切ログに出さないが、
「いつ・どの種類の秘密が消えたか」という事実も残っていない。
値を出さずに対象名（api_key 等）だけをログに残せば、
監査性を高めつつ秘密を漏らさない。

### 成果物

- `desktop/src/commands/config.rs`: 変更（ログ追加）
- `docs/decisions/0135-secret-clear-audit-log.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. 実際に消えた対象名のみをログに残す（値・パス・キーは出さない）。
2. no-op のときはログを出さない。
3. ログは `log` クレート（`log::info!` 等、既存依存）を使う。
4. 既存の 240 テスト + wasm 3、E2E 229 項目を壊さない。

## Acceptance criteria

- 6 ゲート + audit + wasm-test + E2E が成功する。

## Constraints

- 新規依存を追加しない。
- ログに秘密の値を含めない。
- `master` へ push する。force push はしない。
