# Goal

Web 版 EPUB リーダーで、章の切り替えをスクリーンリーダーへ通知する。

## Scope

### 背景

ADR-0021 までで ARIA 属性を付けたが、章を切り替えてもスクリーンリーダーへ
通知が行かず、読み上げユーザーは本文が変わったことに気づきにくい
（ADR-0021 の未決定「スクリーンリーダーでの本文読み上げの最適化」）。

### 成果物

- `web/src/lib.rs`: 変更（章切り替えの通知領域）
- `web/index.html`: 変更（ visually hidden な通知領域のスタイル）
- `tools/verify-web-reader.py`: 変更（通知領域の E2E 検証を追加）
- `docs/decisions/0026-screen-reader-announcements.md`: 追加（決定と検証の記録）
- `GOAL.md`: 本契約

### 必須事項

1. 通知領域:
   - 本文領域の外に `aria-live="polite"`・`aria-atomic="true"` の領域を置く。
   - 章が切り替わるたびに `Chapter N: <title>` を通知する。
   - 初回表示では通知しない（空文字で始める）。
   - 視覚的には非表示（`.sr-only`）にする。
2. 既存の挙動を変えない:
   - 既存の 119 テスト、E2E 101 項目、読書位置、履歴、表示設定、掃除、
     キーボード操作、タッチ操作を変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - 章を移動すると通知領域のテキストが `Chapter ...` になる。
  - 初回表示では通知領域が空である。
  - 本文領域の `aria-live` が維持される。
- 既存の 119 テストと E2E 101 項目が通る。

## Acceptance criteria

- 上記の必須事項がコードと E2E で確認できる。
- 次がすべて成功する。
  - `cargo fmt --all -- --check`
  - `cargo check --all-targets --locked`
  - `cargo test --all-targets --locked`
  - `cargo clippy --all-targets --locked -- -D warnings`
  - `cargo check -p openrustbooks-core --target wasm32-unknown-unknown --locked`
  - `cargo check -p openrustbooks-web --target wasm32-unknown-unknown --locked`
  - `trunk build --offline --dist <一時出力先>`（web）
  - `python3 tools/verify-web-reader.py --dist <出力先>`（E2E）
- CI（`Seduce0068/workflows/rust-gates.yml@v1`）が成功する。

## Constraints

- 新規依存を追加しない。外部送信を追加しない。
- `desktop/` を変更しない。
- プロダクションコードで `unwrap()` / `expect()` / `panic!` / `todo!` を
  使わない。`unsafe` を使わない。
- コメントと doc コメントは日本語で簡潔に書く。
- 無関係なリファクタリングをしない。
- `master` へ push する。force push はしない。
