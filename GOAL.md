# Goal

既知脆弱性の検出をローカル検証ゲートに追加する（todo.md B03 の残り）。

## Scope

### 背景

`cargo audit` で脆弱性 0 件を確認済みだが、`scripts/verify.sh` には
ゲートが無く、CI も billing issue で実行できない。再利用可能な
ローカルゲートを追加し、依存更新時の回帰を防ぐ。

### 成果物

- `scripts/verify.sh`: 変更（cargo audit ゲート追加）
- `docs/decisions/0125-audit-gate.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `cargo audit` をオフライン／キャッシュ優先で実行できるオプションを確認する。
2. 脆弱性 0 件をゲート化する（`--deny warnings` は使わない。
   既存の unmaintained 警告 10 件は baseline）。
3. `cargo-audit` が未インストールの場合はゲートをスキップし警告を出す
   （環境依存で全員の検証を止めない）。
4. `bash -n` / `shellcheck` / `shfmt -d` と `./scripts/verify.sh` が成功する。

## Acceptance criteria

- 6 ゲート + audit + wasm + E2E すべて成功する。

## Constraints

- 新規依存を追加しない（`cargo-audit` は既存インストール済みツール）。
- CI ワークフローは変更しない。
- `master` へ push する。force push はしない。
