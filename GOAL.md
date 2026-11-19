# Goal

MSRV を 1.95 と宣言し CI で検証する（R6 TD-M15、ADR-0214）。

## Scope

### 成果物

- workspace / 3 member: rust-version 1.95
- ci.yml: rust_toolchain 1.95.0
- core/src/epub.rs: clippy 1.95 対応
- docs/decisions/0214-msrv.md / README / todo / 索引 / log

### 決定

- 現行ロックの依存要件（parking_lot / lock_api = 1.95）に合わせる。
- Cargo.lock 未変更、共通ワークフロー未変更。

## Acceptance criteria

- 1.95 の全ゲートと stable verify.sh（253/0）成功（検証済み）。
