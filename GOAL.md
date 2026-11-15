# Goal

Web 版の読書位置インポートで、既存データが新しいときは上書きしない
（マージ導入）。

## Scope

### 背景

ADR-0026・ADR-0030 まででインポートは同キーを常に上書きする。古い
バックアップを読み込むと、現在の読書位置が古い値で壊れる
（ADR-0026 の未決定「インポート時のマージ（上書きではなく統合）」）。

### 成果物

- `web/src/lib.rs`: 変更（位置インポートのマージ）
- `tools/verify-web-reader.py`: 変更（マージの E2E 検証を追加）
- `docs/decisions/0032-import-merge.md`: 追加（決定と検証の記録）
- `GOAL.md`: 本契約

### 必須事項

1. 位置のマージ:
   - インポートする位置キーが localStorage に既に存在する場合、
     `savedAt` を比較して新しい方を採用する。
   - 既存の `savedAt` の方が新しい（または等しい）場合は既存を維持し、
     取り込み件数に数えない。
   - 既存の値が `SavedPosition` として読めない場合は、インポート値で
     上書きする（現在の挙動と同じ）。
   - `settings` と `history` は従来どおりインポート値で上書きする。
2. 既存の挙動を変えない:
   - 既存の 119 テスト、E2E 121 項目、読書位置、履歴、表示設定、掃除、
     キーボード操作、タッチ操作、landmarks、page-list、履歴一覧、
     履歴クリア、エクスポート・インポートを変えない。
   - 既存キーが無い位置は従来どおり取り込む。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - 既存の `savedAt` が新しい場合、インポートで上書きされない。
  - 既存キーが無い位置は取り込まれる。
- 既存の 119 テストと E2E 121 項目が通る。

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
