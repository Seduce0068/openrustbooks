# Goal

`core/Cargo.toml` の rumdl `default-features = false` を静的ゲートで保護する（R6 TD-M06）。

## Scope

### 背景

rumdl の feature は Cargo.lock に記録されないため、
`core/Cargo.toml` の 2 行（native / wasm）が唯一の担保。
将来 `default-features = true` に戻ると tower-lsp / notify / jemalloc 等が
再加入し、CI ディスク・ビルドが劣化しても lock 差分で気付けない。

### 成果物

- `scripts/verify.sh`: 変更（rumdl feature 静的検査ゲート追加）
- `docs/decisions/0195-rumdl-feature-gate.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. `core/Cargo.toml` の rumdl 宣言が 2 行とも `default-features = false` であることを検査する。
2. 宣言が無い／期待と異なる場合は fail-closed でゲート失敗。
3. 既存の `no-external-font-refs` ゲートと同じ `run` ヘルパー形式を使う。
4. bash -n / shellcheck / shfmt -d / ./scripts/verify.sh（E2E 250）が成功。
5. ネガティブテスト（一時的な改変 Cargo.toml で rc=1）を確認。

## Acceptance criteria

- 全ゲート成功、E2E 250 項目成功。

## Constraints

- 新規依存を追加しない。
- `core/Cargo.toml` 自体は変更しない（期待値の固定のみ）。
- `master` へ push する。force push はしない。
