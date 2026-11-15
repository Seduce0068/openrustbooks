# Goal

Web 版 EPUB リーダーに page-list（ページ番号）による移動 UI を追加する。

## Scope

### 背景

core は `page_list()` を公開しているが、Web UI は使っていない
（ADR-0009・ADR-0027 の未決定「page-list の活用（ページ番号での移動 UI）」）。

### 成果物

- `web/src/lib.rs`: 変更（ページ番号入力と移動）
- `tools/verify-web-reader.py`: 変更（page-list の E2E 検証を追加、サンプル EPUB に page-list を追加）
- `docs/decisions/0028-page-list-navigation.md`: 追加（決定と検証の記録）
- `GOAL.md`: 本契約

### 必須事項

1. ページ移動 UI:
   - ツールバーにページ番号入力（`<input id="reader-page-input">`）と移動
     ボタン（`<button id="reader-page-go">`）を置く。
   - `WasmEpubBook::page_list()` で取得する。
   - 入力したページ番号に一致する entry を探し、その章へ移動し、
     フラグメントがあればスクロールする。
   - 一致する entry が無い場合は状態メッセージで報告する。
   - page-list が空のときは入力とボタンを disabled にする。
2. ARIA:
   - 入力に `aria-label="Page"` を付ける。
3. 既存の挙動を変えない:
   - 既存の 119 テスト、E2E 106 項目、読書位置、履歴、表示設定、掃除、
     キーボード操作、タッチ操作、landmarks を変えない。

### 必須テスト

- `tools/verify-web-reader.py` のサンプル EPUB に page-list を追加する。
- `tools/verify-web-reader.py` に追加する。
  - ページ入力が存在する。
  - ページ番号を入力して移動すると、該当章へ移動する。
  - 存在しないページ番号では状態メッセージが出る。
- 既存の 119 テストと E2E 106 項目が通る。

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
- CI は billing issue で実行不可（ローカル検証で代替）。

## Constraints

- 新規依存を追加しない。保存先は localStorage とメモリ。外部送信を追加しない。
- `desktop/` を変更しない。
- プロダクションコードで `unwrap()` / `expect()` / `panic!` / `todo!` を
  使わない。`unsafe` を使わない。
- コメントと doc コメントは日本語で簡潔に書く。
- 無関係なリファクタリングをしない。
- `master` へ push する。force push はしない。
