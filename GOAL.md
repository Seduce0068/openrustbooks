# Goal

画像寸法取得クレート `imagesize` の wasm32 互換性を検証する（ADR-0136 の未決定）。

## Scope

### 背景

ADR-0136 は PNG/JPEG/WebP/GIF の寸法取得に `imagesize 0.15.0`（MIT・軽量）を第一候補としたが、
wasm32-unknown-unknown でのビルド可否は未確認。ここを解消しないと
段階導入の判断ができない。

### 成果物

- `docs/decisions/0137-imagesize-wasm-compat.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. リポジトリ外の一時ディレクトリに最小クレートを作り、
   `imagesize = { version = "0.15", default-features = false, features = ["png","jpeg","gif","webp"] }`
   を wasm32 ターゲットでビルドする。
2. `imagesize::blob_size(&bytes)` が wasm で使えることを型チェックで確認する。
3. 結果（成功／失敗と理由）を ADR に記録する。
4. リポジトリの Cargo.toml / Cargo.lock は変更しない（隔離検証）。

## Acceptance criteria

- 判断が ADR に記録される。
- markdownlint が通る。

## Constraints

- リポジトリに新規依存を追加しない（調査のみ）。
- 一時ファイルは検証後に削除する。
- `master` へ push する。force push はしない。
