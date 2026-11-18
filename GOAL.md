# Goal

未使用の workspace.dependencies.rumdl を削除する（ADR-0155 / 0163 の未決定）。

## Scope

### 背景

`Cargo.toml:27` に `rumdl = { version = "0.3", default-features = true }` が残るが、
core は個別に `default-features = false` を指定しており、
どの member も `workspace = true` で rumdl を参照していない。
将来誤って workspace 版を使うと default feature（tower-lsp / jemalloc 等）が復活する罠がある。

### 成果物

- `Cargo.toml`: 変更（workspace.dependencies から rumdl を削除）
- `docs/decisions/0168-workspace-rumdl-cleanup.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. workspace.dependencies から rumdl 行を削除する。
2. core / desktop / web の Cargo.toml は変更しない（個別指定を維持）。
3. cargo check / test / clippy / wasm check / audit / verify.sh（E2E 242）が通る。
4. ADR-0155 / 0163 の未決定を解消する。

## Acceptance criteria

- 全ゲート成功、E2E 242 項目成功。

## Constraints

- core の rumdl featureless 指定は維持する。
- 新規依存を追加しない。
- `master` へ push する。force push はしない。
