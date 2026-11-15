# Goal

Web 版のエクスポート・インポートに移動履歴を含める。

## Scope

### 背景

ADR-0026 まででエクスポート形式は `{ positions, settings }`。履歴は
ADR-0024 で localStorage に永続化されているが、バックアップの対象に
含まれない（ADR-0026・ADR-0029 の未決定「履歴のエクスポート・インポート
への含有」）。

### 成果物

- `web/src/lib.rs`: 変更（エクスポート・インポートの形式拡張）
- `tools/verify-web-reader.py`: 変更（履歴のエクスポート・インポートの E2E 検証を追加）
- `docs/decisions/0030-history-export-import.md`: 追加（決定と検証の記録）
- `GOAL.md`: 本契約

### 必須事項

1. エクスポート:
   - 出力 JSON に `history` を追加する。
     `{ "positions": {...}, "settings": {...}, "history": { "<履歴キー>": { "entries": [...], "index": n }, ... } }`
   - `history` には `openrustbooks:history:` で始まる全キーを含める。
   - 値は既存の `SavedHistory`（entries / index）と同じ形にする。
2. インポート:
   - `history` キーがあれば各履歴を取り込む。
     - 値が `SavedHistory` として読めるものだけ取り込む（fail-closed）。
   - `history` が無い形式（旧形式）でも従来どおり positions / settings だけ
     取り込む。
   - 取り込んだ履歴件数を状態メッセージに含める。
3. 既存の挙動を変えない:
   - 既存の 119 テスト、E2E 114 項目、読書位置、履歴、表示設定、掃除、
     キーボード操作、タッチ操作、landmarks、page-list、履歴一覧を変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - エクスポート JSON に `history` が含まれる。
  - 新形式のインポートで履歴が復元される。
- 既存の 119 テストと E2E 114 項目が通る。

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

- 新規依存を追加しない。保存先は localStorage とブラウザのダウンロード。
  外部送信を追加しない。
- `desktop/` を変更しない。
- プロダクションコードで `unwrap()` / `expect()` / `panic!` / `todo!` を
  使わない。`unsafe` を使わない。
- コメントと doc コメントは日本語で簡潔に書く。
- 無関係なリファクタリングをしない。
- `master` へ push する。force push はしない。
