# Goal

Web 版 EPUB リーダーのエクスポート・インポートに表示設定を含める。

## Scope

### 背景

ADR-0022 までで読書位置のみを JSON で書き出し・読み込みできるが、表示設定
（文字サイズ・行間・ダーク表示）はバックアップの対象に含まれない
（ADR-0018・ADR-0022 の未決定「設定のエクスポート・インポート」）。
エクスポート形式を拡張し、表示設定も含める。

### 成果物

- `web/src/lib.rs`: 変更（エクスポート・インポートの形式拡張）
- `tools/verify-web-reader.py`: 変更（設定のエクスポート・インポートの E2E 検証を追加）
- `docs/decisions/0026-settings-export-import.md`: 追加（決定と検証の記録）
- `GOAL.md`: 本契約

### 必須事項

1. エクスポート:
   - 出力 JSON は `{ "positions": { ... }, "settings": { ... } }` の形式にする。
   - 旧形式（`{ position_key: value }`）との区別のため、新しい形式では
     トップレベルに `positions` と `settings` を置く。
   - `positions` には `openrustbooks:position:` で始まる全キーを含める。
   - `settings` には現在の表示設定を含める。
2. インポート:
   - 新形式（`positions` / `settings` キーを持つ）と旧形式（トップレベルに
     position キーが並ぶ）の両方を受け付ける。
   - 新形式では `positions` と `settings` をそれぞれ取り込む。
   - 旧形式では position キーのみを取り込む（表示設定は触らない）。
   - 不正な JSON は拒否する。
   - 取り込んだ position 件数と settings の有無を状態メッセージに表示する。
3. 既存の挙動を変えない:
   - 既存の 119 テスト、E2E 101 項目、読書位置、履歴、表示設定、掃除、
     キーボード操作、タッチ操作を変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - 新形式のエクスポートに `positions` と `settings` が含まれる。
  - 新形式のインポートで表示設定が復元される。
  - 旧形式の JSON もインポートできる。
  - 不正な JSON は拒否される。
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

- 新規依存を追加しない。保存先は localStorage とブラウザのダウンロード。
  外部送信を追加しない。
- `desktop/` を変更しない。
- プロダクションコードで `unwrap()` / `expect()` / `panic!` / `todo!` を
  使わない。`unsafe` を使わない。
- コメントと doc コメントは日本語で簡潔に書く。
- 無関係なリファクタリングをしない。
- `master` へ push する。force push はしない。
