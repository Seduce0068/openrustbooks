# Goal

core の `init()` 二重定義を解消し、Wasm 初期化を一本化する（todo.md D03）。

## Scope

### 背景

- `core/src/lib.rs` に `pub fn init()`（native では tracing_subscriber、
  wasm32 では panic hook + tracing_wasm）
- `core/src/wasm.rs` に `#[wasm_bindgen] pub fn init()`（JS から呼ばれる）
- wasm32 では `pub use wasm::*` により名前が衝突し得るが、
  現在は Rust 側（`web/src/lib.rs`）が `openrustbooks_core::init` を呼ぶと
  lib.rs の版が使われ、JS 側の wasm-bindgen 版とは別実装になっている。

### 成果物

- `core/src/lib.rs`: 変更（Wasm 用 init を wasm.rs の実装へ委譲）
- `core/src/wasm.rs`: 変更（初期化処理を共通関数へ集約）
- `GOAL.md`: 本契約

### 必須事項

1. 初期化処理（panic hook + tracing_wasm）を一箇所に集約する。
2. `core::init()` と wasm-bindgen の `init()` の両方が同じ処理を呼ぶ。
3. native の tracing_subscriber 初期化は現状維持。
4. 既存の 126 テスト、E2E 220 項目を変えない。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- 新規依存を追加しない。
- `desktop/` を変更しない。
- `master` へ push する。force push はしない。
