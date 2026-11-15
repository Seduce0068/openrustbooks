# Goal

Web 版 EPUB リーダーで、保存した読書位置を JSON として書き出し・読み込み
できるようにする。

## Scope

### 背景

ADR-0019 までで読書位置を localStorage に保存・掃除するようにしたが、
バックアップや別ブラウザへの移行の手段が無い（ADR-0019 の未決定「読書位置の
エクスポート・インポート」）。ブラウザ内で完結する JSON の書き出し・
読み込みを追加する。

### 成果物

- `web/src/lib.rs`: 変更（エクスポート・インポートのボタンと処理）
- `tools/verify-web-reader.py`: 変更（エクスポート・インポートの E2E 検証を追加）
- `docs/decisions/0022-position-export-import.md`: 追加（決定と検証の記録）
- `GOAL.md`: 本契約

### 必須事項

1. エクスポート:
   - `Export` ボタンで `openrustbooks:position:` で始まる全キーを
     `{ key: value }` の JSON オブジェクトにまとめ、
     `openrustbooks-positions.json` としてダウンロードする。
   - ダウンロードは `<a download>` と `URL.createObjectURL` を使う。
   - 削除されたキー（掃除済み）は含まれない。
2. インポート:
   - `Import` ボタン（`<input type="file" accept=".json">`）で JSON を読み込む。
   - 形式が `{ key: value }` のオブジェクトで、キーが
     `openrustbooks:position:` で始まるものだけを取り込む。
   - 既存のキーは上書きする。
   - 取り込んだ件数を状態メッセージに表示する。
   - 形式が不正な場合は状態メッセージで報告し、何も取り込まない。
   - `openrustbooks:settings` はインポートの対象にしない。
3. 既存の挙動を変えない:
   - 既存の 119 テスト、E2E 86 項目、読書位置、履歴、表示設定、掃除、
     キーボード操作を変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - `Export` でダウンロードが発生し、内容が `{ key: value }` である。
  - `Import` で JSON を読み込み、読書位置が復元される。
  - 不正な JSON では何も取り込まず、状態メッセージが出る。
  - インポート後も表示設定は残る。
- 既存の 119 テストと E2E 86 項目が通る。

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

- 新規依存を追加しない。保存先は localStorage とブラウザのダウンロード。
  外部送信を追加しない。
- `desktop/` を変更しない。
- プロダクションコードで `unwrap()` / `expect()` / `panic!` / `todo!` を
  使わない。`unsafe` を使わない。
- コメントと doc コメントは日本語で簡潔に書く。
- 無関係なリファクタリングをしない。
- `master` へ push する。force push はしない。
