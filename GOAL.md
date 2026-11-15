# Goal

Web 版 EPUB リーダーに landmarks（表紙・本文開始など）の移動 UI を追加する。

## Scope

### 背景

core は `landmarks()` と `page_list()` を公開しているが、Web UI は目次しか
使っておらず、landmarks の位置へ直接移動できない（ADR-0009・ADR-0010 の未決定
「landmarks / page-list の活用」）。

### 成果物

- `web/src/lib.rs`: 変更（landmarks ドロップダウンと移動）
- `tools/verify-web-reader.py`: 変更（landmarks の E2E 検証を追加、サンプル EPUB に landmarks を追加）
- `docs/decisions/0027-landmarks-ui.md`: 追加（決定と検証の記録）
- `GOAL.md`: 本契約

### 必須事項

1. landmarks ドロップダウン:
   - ツールバーに `<select id="reader-landmarks">` を置く。
   - `WasmEpubBook::landmarks()` で取得し、各エントリの title をラベルに使う。
   - landmarks が空のときは disabled にする。
   - 選択を変えると、その landmark の章へ移動し、フラグメントがあれば
     スクロールする。
2. ARIA:
   - ドロップダウンに `aria-label="Landmarks"` を付ける。
3. 既存の挙動を変えない:
   - 既存の 119 テスト、E2E 103 項目、読書位置、履歴、表示設定、掃除、
     キーボード操作、タッチ操作を変えない。

### 必須テスト

- `tools/verify-web-reader.py` のサンプル EPUB に landmarks を追加する。
- `tools/verify-web-reader.py` に追加する。
  - landmarks ドロップダウンが存在する。
  - landmarks が空でないとき、選択で章が移動する。
- 既存の 119 テストと E2E 103 項目が通る。

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
