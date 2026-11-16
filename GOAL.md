# Goal

Web 版のエクスポート JSON に形式バージョンを付与し、未知のバージョンを安全に拒否する。

## Scope

### 背景

ADR-0026・ADR-0030 まででエクスポート形式は
`{ positions, settings, history }` だが、バージョン情報が無い。将来形式が
変わったときに、古いリーダーが新しいファイルを誤って取り込み、
読書位置や履歴を壊す恐れがある（ADR-0030・ADR-0032 の未決定
「エクスポート形式のバージョニング」）。

### 成果物

- `web/src/lib.rs`: 変更（バージョン定数、エクスポートへの付与、インポートでの検証）
- `tools/verify-web-reader.py`: 変更（バージョン検証の E2E 検証を追加）
- `docs/decisions/0036-export-versioning.md`: 追加（決定と検証の記録）
- `GOAL.md`: 本契約

### 必須事項

1. エクスポート:
   - 出力 JSON のトップレベルに `"version": 1` を付与する。
   - 既存の `positions` / `settings` / `history` はそのまま維持する。
2. インポート:
   - `version` が無い JSON（旧形式）は従来どおり取り込む。
   - `version` が 1 の JSON は従来どおり取り込む。
   - `version` が 1 以外（未知のバージョン）の JSON は拒否し、
     状態メッセージで理由を表示する。何も取り込まない（fail-closed）。
3. 既存の挙動を変えない:
   - 既存の 119 テスト、E2E 136 項目、読書位置、履歴、表示設定、掃除、
     キーボード操作、タッチ操作、landmarks、page-list、履歴一覧、
     履歴クリア・個別削除、エクスポート・インポート、マージ、
     カラースキーム追従、画像の非同期デコードを変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - `version: 1` の JSON がインポートできる。
  - 未知の `version`（例: 999）の JSON は拒否され、現在のデータが
    壊れない。
- 既存の 119 テストと E2E 136 項目が通る。

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
