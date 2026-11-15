# Goal

Web 版 EPUB リーダーの履歴一覧から、選択中のエントリを個別に削除できるようにする。

## Scope

### 背景

ADR-0031 までで履歴の全消去（クリア）はできるが、任意の 1 件だけを消す手段が
無い（ADR-0029・ADR-0031 の未決定「履歴エントリの個別削除」）。

### 成果物

- `web/src/lib.rs`: 変更（履歴の個別削除ボタンと処理）
- `tools/verify-web-reader.py`: 変更（個別削除の E2E 検証を追加）
- `docs/decisions/0035-history-delete-entry.md`: 追加（決定と検証の記録）
- `GOAL.md`: 本契約

### 必須事項

1. 個別削除:
   - ツールバーに `<button id="reader-history-delete">` を置く。
   - クリックで**現在選択中の履歴エントリ**（`history_index` が指すもの）を
     履歴から削除する。
   - 削除後も表示中の章・スクロール位置は変えない（再描画しない）。
   - 削除後の `history_index` は次のとおり調整する。
     - 削除した位置より後ろにエントリがある場合は同じインデックス
       （次のエントリが繰り上がる）。
     - 削除した位置が最後の場合は 1 つ前のインデックス。
   - localStorage の当該本の履歴キーも更新する。
   - 履歴が 1 件のときは disabled にする（履歴を空にしない）。
2. ARIA:
   - ボタンに `aria-label="Delete history entry"` を付ける。
3. 既存の挙動を変えない:
   - 既存の 119 テスト、E2E 131 項目、読書位置、履歴、表示設定、掃除、
     キーボード操作、タッチ操作、landmarks、page-list、履歴一覧、
     履歴クリア、エクスポート・インポート、マージ、カラースキーム追従、
     画像の非同期デコードを変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - 履歴 3 件の状態で削除を押すと 2 件になる。
  - 削除後も表示中の章は変わらない。
  - 履歴が 1 件のときは削除ボタンが disabled。
- 既存の 119 テストと E2E 131 項目が通る。

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
