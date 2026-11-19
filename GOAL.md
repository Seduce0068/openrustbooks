# Goal

MSRV を 1.95 と宣言し、CI で実際に検証する（R6 TD-M15）。

## Scope

### 背景

`Cargo.toml` に `rust-version` が無く、CI も stable に依存していた。
`cargo metadata` の依存 MSRV を確認すると parking_lot 0.13.0 / lock_api 0.5.0 が
**1.95** を要求し、tauri 2.12.5 は 1.90、rumdl 0.3.2 は 1.94 だった。
1.95 ツールチェーンで fmt / check / test は成功したが、
clippy 1.95 が `core/src/epub.rs:1472` の `collapsible_match` を
`-D warnings` で失敗させた。

### 成果物

- `Cargo.toml`: 変更（`rust-version = "1.95"`）
- `core/Cargo.toml` / `desktop/Cargo.toml` / `web/Cargo.toml`: 変更（`rust-version.workspace = true`）
- `.github/workflows/ci.yml`: 変更（`rust_toolchain: 1.95.0`）
- `core/src/epub.rs`: 変更（clippy 1.95 対応）
- `docs/decisions/0214-msrv.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. workspace と 3 member に rust-version 1.95 を宣言する。
2. CI の `rust_toolchain` を 1.95.0 に設定する。
3. `rustup run 1.95.0` で fmt / check / test / clippy が成功する。
4. 既存 stable（1.100）の `./scripts/verify.sh`（E2E 253）も成功する。
5. ADR に MSRV 決定の根拠（依存要件と実測）を記録する。

## Acceptance criteria

- 1.95 の全ゲートと stable の verify.sh（253/0）が成功する。

## Constraints

- 依存バージョンは変更しない（MSRV は現状のロックに合わせる）。
- 共通ワークフロー自体は変更しない。
- `master` へ push する。force push はしない。
