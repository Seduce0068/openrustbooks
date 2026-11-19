# Goal

CI / ローカルゲートの整合を確認し、`trunk build --locked` を追加する（R6 R3-ci#3,#5）。

## Scope

### 背景

`wasm_targets` は共通ワークフローで target 導入のみを行い、
実検査は `extra_gates` が担うため二重実行ではない。
CI の `CARGO_PROFILE_*` 環境変数は GitHub runner のディスク対策で、
ローカル verify.sh に全局適用すべきではない。
一方 `scripts/verify.sh` の `trunk build --offline` に `--locked` が無く、
`Cargo.lock` を黙って更新できる fail-open が残っていた。

### 成果物

- `scripts/verify.sh`: 変更（`trunk build --offline --locked --dist ...`）
- `docs/decisions/0237-ci-gate-consistency.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. `trunk build` に `--locked` を追加する（`--offline` は維持）。
2. `ci.yml` / `ci-wasm-test.sh` は変更しない（二重実行ではないため）。
3. ADR に wasm_targets と extra_gates の役割分担、
   ビルドプロファイル差、env をローカルへ適用しない理由を記録する。
4. markdownlint 0 issues、verify.sh（E2E 254）成功。

## Acceptance criteria

- verify.sh が `--locked` 付きで全ゲート成功。

## Constraints

- 共通ワークフロー自体は変更しない。
- 外部サービス・billing の状態は未決定として残す。
- `master` へ push する。force push はしない。
