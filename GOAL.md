# Goal

全ゲート・Wasm ビルド・E2E を一括実行する検証スクリプトを追加する。

## Scope

### 背景

調査レポートで「E2E・`trunk build`・`cargo audit` が CI ゲート外」と指摘された。
リポジトリ内には検証スクリプトが無く、`/tmp/openrustbooks-gates.sh` が
一時的に使われているだけ。再現可能な `scripts/verify.sh` を追加する。

事前調査: `find . -maxdepth 3 -type f \( -name '*.sh' -o -name 'Makefile' -o -name 'justfile' \)`
の結果、リポジトリ内に既存の検証スクリプトは無い。

### 成果物

- `scripts/verify.sh`: 追加
- `docs/decisions/0084-verification-script.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `scripts/verify.sh` は次を順に実行する。
   - 6 ゲート（fmt / check / test / clippy / wasm32 core / wasm32 web）
   - `trunk build --offline`（一時ディレクトリへ出力）
   - `python3 tools/verify-web-reader.py --dist <出力先>`
2. 終了時に一時ディレクトリを削除する（trap）。
3. `bash -n` と `shellcheck`（利用可能なら）が通る。
4. スクリプト自体を実行して全項目が成功する。

## Acceptance criteria

- `scripts/verify.sh` が成功する。
- 6 ゲートすべて成功する。

## Constraints

- 新規依存を追加しない（既存ツールの組み合わせのみ）。
- `desktop/` を変更しない。
- コメントは日本語で簡潔に書く。
- `master` へ push する。force push はしない。
