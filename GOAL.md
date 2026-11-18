# Goal

未使用の pulldown-cmark 依存を削除し、依存ツリーを整理する（ADR-0162 の未決定）。

## Scope

### 背景

`Cargo.toml:26` と `core/Cargo.toml:21` で pulldown-cmark 0.13（`simd` feature 付き）を宣言しているが、
コードからは未使用（ADR-0162 で comrak を採用）。
`simd` feature は wasm で恩恵が薄く、過去に脆弱性修正（ADR-0086）もある。

### 成果物

- `Cargo.toml`: 変更（workspace 依存から削除）
- `core/Cargo.toml`: 変更（依存から削除）
- `Cargo.lock`: 変更（未使用依存の除去）
- `docs/decisions/0163-remove-unused-markdown-dep.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. 両マニフェストから pulldown-cmark を削除する。
2. `cargo check --all-targets --locked` / test / clippy / wasm check が通ることを確認（必要なら Cargo.lock 更新）。
3. `cargo audit --no-fetch` の警告数が baseline 10 から減るか確認（減らない場合も記録）。
4. E2E 238 項目と全ゲートを壊さない。

## Acceptance criteria

- 全ゲート成功、E2E 238 項目成功。

## Constraints

- comrak は維持する。
- 新規依存を追加しない。
- `master` へ push する。force push はしない。
