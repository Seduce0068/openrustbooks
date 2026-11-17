# Goal

wasm32 単体テストの実行手段を確立する（ADR-0117 の未決定、test-scanner #14）。

## Scope

### 背景

`core/src/wasm.rs` に wasm32 限定テストを追加したが、
`scripts/verify.sh` の wasm32 ゲートは `cargo check` のみで、
テストは実行されていない。`wasm-bindgen-test` は dev-dep にあり、
`wasm-pack` もインストール済み。

### 成果物

- `scripts/verify.sh`: 変更（wasm32 テスト実行を追加、可能な場合）
- `docs/decisions/0119-wasm-test-runner.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. wasm32 テストを実際に実行できるコマンドを特定する。
2. 実行可能なら `scripts/verify.sh` にゲートとして追加する。
3. 実行不能ならその理由を ADR に記録し、現状の `cargo check` ゲートを維持する。
4. 既存の 195 テスト、E2E 228 項目を壊さない。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- 新規依存を追加しない。
- 既存の wasm32 check ゲートは維持する。
- `master` へ push する。force push はしない。
