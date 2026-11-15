# Goal

Web 版 EPUB リーダーのダーク表示を OS の prefers-color-scheme に追従させる。

## Scope

### 背景

ADR-0018・ADR-0023 まででダーク表示は手動トグルのみ。OS の
prefers-color-scheme: dark を検知して自動でダーク表示に切り替える手段が無い
（ADR-0018・ADR-0023 の未決定「テーマの自動切替（prefers-color-scheme）」）。

### 成果物

- `web/src/lib.rs`: 変更（prefers-color-scheme の検知と自動切替）
- `web/index.html`: 変更（色スキームのメタタグ）
- `tools/verify-web-reader.py`: 変更（自動切替の E2E 検証を追加）
- `docs/decisions/0033-prefers-color-scheme.md`: 追加（決定と検証の記録）
- `GOAL.md`: 本契約

### 必須事項

1. 自動切替:
   - ダーク表示が「自動」（auto）の状態を追加する。
   - OS の prefers-color-scheme: dark を検知して、自動モードのときは
     ダーク表示にする。
   - OS の設定が変わったときにリアルタイムで追従する。
   - ユーザーが手動でダーク/ライトを選んだときは自動モードを解除する。
2. 設定の永続化:
   - 自動/手動の選択を localStorage に保存する。
   - 再読み込み後も選択が復元される。
3. 既存の挙動を変えない:
   - 既存の 119 テスト、E2E 123 項目、読書位置、履歴、表示設定、掃除、
     キーボード操作、タッチ操作、landmarks、page-list、履歴一覧、
     履歴クリア、エクスポート・インポート、マージを変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - ダーク表示の自動切替が設定できる。
  - 設定の保存と復元が動作する。
- 既存の 119 テストと E2E 123 項目が通る。

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

- 新規依存を追加しない。保存先は localStorage。外部送信を追加しない。
- `desktop/` を変更しない。
- プロダクションコードで `unwrap()` / `expect()` / `panic!` / `todo!` を
  使わない。`unsafe` を使わない。
- コメントと doc コメントは日本語で簡潔に書く。
- 無関係なリファクタリングをしない。
- `master` へ push する。force push はしない。
