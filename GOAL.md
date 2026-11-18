# Goal

`wasm-pack test --node` を CI のマージゲートへ追加する（R6 TD-M05）。

## Scope

### 背景

`verify.sh` は wasm-pack test を実行するが、
CI は `rust-gates.yml@v1` に委譲しており `extra_gates` には
wasm32 check 2 行しか無い。共通ワークフローは wasm-pack を導入しないため、
wasm-bindgen-test 10 件が CI で実行されていない（ADR-0119 の未決定）。

### 成果物

- `scripts/ci-wasm-test.sh`: 新規（wasm-pack 導入＋テスト実行）
- `scripts/verify.sh`: 既存 wasm-test ゲートから共通スクリプトを呼ぶ
- `.github/workflows/ci.yml`: `extra_gates` に共通スクリプトを追加
- `docs/decisions/0196-ci-wasm-test-gate.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. wasm-pack が無い場合のみ `cargo install wasm-pack --locked --version 0.13.1` で導入する。
2. `core/` で `wasm-pack test --node --locked` を実行する。
3. verify.sh の wasm-test ゲートは同じスクリプトを呼び、単一情報源にする。
4. bash -n / shellcheck / shfmt -d / ./scripts/verify.sh（E2E 250）が成功。
5. CI YAML が parse でき、`extra_gates` に新スクリプトが入ること。

## Acceptance criteria

- 全ゲート成功、E2E 250 項目成功。

## Constraints

- 共通ワークフロー（外部リポジトリ）は変更しない。
- 新規依存を Cargo に追加しない（CI ツール導入のみ）。
- `master` へ push する。force push はしない。
