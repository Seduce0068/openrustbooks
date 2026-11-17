# Goal

ゲート数を「6 ゲート」に統一する（todo.md A09 / R3-ci#1）。

## Scope

### 背景

実際の検証は fmt / check / test / clippy / wasm32 core / wasm32 web の
6 項目だが、README:149 と `.github/workflows/ci.yml:3` のコメントに
「5 ゲート」と残っている。README:167 と `scripts/verify.sh` は 6 ゲート。

### 成果物

- `README.md`: 変更（5 → 6）
- `.github/workflows/ci.yml`: 変更（コメントの 5 → 6）
- `GOAL.md`: 本契約

### 必須事項

1. 両ファイルの「5 ゲート」を「6 ゲート」に変更する。
2. コードの挙動・CI 設定本体は変更しない（コメントのみ）。
3. 6 ゲート・E2E が通る。

## Acceptance criteria

- ドキュメント内のゲート数表記が 6 に統一される。
- 6 ゲート・E2E が成功する。

## Constraints

- CI の `- uses:` や入力 parameters は変更しない。
- `master` へ push する。force push はしない。
