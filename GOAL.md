# Goal

core の Wasm バインディング本編を責務別モジュールへ分割する（R6 R2-crates#4）。

## Scope

### 背景

ADR-0257 で tests 379 行を `wasm/tests.rs` へ切り出し、
`core/src/wasm.rs` は 537 行（本編）になった。
残る本編は epub / archive / lint / 共有基盤が混在しており、
公開パス（`pub use wasm::*`）と native 向け cfg を維持しつつ
子モジュールへ分離する。

### 成果物

- `core/src/wasm.rs`: 変更（共有基盤・再エクスポート・tests 参照のみ残す）
- `core/src/wasm/epub.rs`: 追加（WasmEpubBook と DTO / serializer）
- `core/src/wasm/archive.rs`: 追加（WasmArchive / 固定エラー / DTO）
- `core/src/wasm/lint.rs`: 追加（WasmMarkdownLinter / lint DTO）
- `docs/decisions/0259-wasm-binding-module-split.md`: 追加
- `todo.md`: R2-crates#4 完了
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. 既存の公開 API（`init` / `image_dimensions_wasm` / `render_markdown_wasm` /
   `sanitize_svg_wasm` / `WasmEpubBook` / `WasmArchive` / `WasmArchiveEntry` /
   `WasmMarkdownLinter` / `WasmLintIssue` / `WasmLintFixResult` / `utils::version`）
   を同じ Rust パスで維持する。
2. native 向け `init()` / `utils::version()` と `cfg_attr` の挙動を維持する。
3. tests は `wasm/tests.rs` の本文を変更せず、必要なら `pub(super)` で定数 / DTO を親へ見せる。
4. fail-closed の固定エラー文言と redaction 契約を変更しない。
5. 全ゲート成功（wasm 14 / core 284 / E2E 255）。

## Acceptance criteria

- wasm 14 / core 284 / E2E 255、全ゲート成功。

## Constraints

- 公開 API・既存 test 本文・E2E・Cargo.toml は変更しない。
- `unsafe` を追加しない。
- `master` へ push する。force push はしない。
