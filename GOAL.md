# Goal

native 側の rumdl 依存を `default-features = false` に絞り、不要な CLI/LSP スタックを除去する（ADR-0151 の残タスク）。

## Scope

### 背景

`core/Cargo.toml:64` は `rumdl = { version = "0.3", default-features = true }`。
rumdl 0.3.2 の default feature は `parallel` + `native` で、
`native` は tower-lsp / tokio / notify / env_logger / blake3 / postcard /
colored / ec4rs 等を引く。CI では tikv-jemalloc-sys のディスク問題を
debuginfo 破棄で回避している（ci.yml:35-41）。
core は `rumdl_lib::config::Config` と rules API のみを使用しており、
LSP / CLI は不要（core/src/ai_markdown/lint.rs:4）。

### 成果物

- `core/Cargo.toml`: 変更（native も `default-features = false`）
- `Cargo.lock`: 変更（不要依存の除去）
- `.github/workflows/ci.yml`: 変更（可能なら env_vars の debuginfo 回避を削除）
- `docs/decisions/0155-rumdl-feature-reduction.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. native の rumdl を `default-features = false` に変更し、
   `cargo check --all-targets --locked` / `cargo test --all-targets --locked` /
   `cargo clippy --all-targets --locked -- -D warnings` が通ることを確認。
2. `serde_yaml 0.9.34+deprecated` がロックから消えるか確認（消えない場合は理由を記録）。
3. `cargo audit --no-fetch` の警告数が baseline 10 から減るか確認（減らない場合も記録）。
4. CI の `env_vars`（debuginfo 回避）が不要なら削除。必要なら維持して理由を記録。
5. E2E 233 項目と全ゲートを壊さない。

## Acceptance criteria

- 全ゲート成功、E2E 233 項目成功。
- 依存削減の効果が ADR に記録される。

## Constraints

- rumdl のバージョンは 0.3 のまま（アップグレードは別タスク）。
- 新規依存を追加しない。
- `master` へ push する。force push はしない。
