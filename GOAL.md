# Goal

Web 版 EPUB リーダーに移動履歴の全消去（クリア）を追加する。

## Scope

### 背景

ADR-0029 までで履歴一覧から任意の位置へジャンプできるが、履歴を消す手段が
無い（ADR-0029 の未決定「履歴エントリの削除（個別削除・全消去）」）。

### 成果物

- `web/src/lib.rs`: 変更（履歴クリアボタンと処理）
- `tools/verify-web-reader.py`: 変更（履歴クリアの E2E 検証を追加）
- `docs/decisions/0031-history-clear.md`: 追加（決定と検証の記録）
- `GOAL.md`: 本契約

### 必須事項

1. 履歴クリア:
   - ツールバーに `<button id="reader-history-clear">` を置く。
   - クリックで開いている本の履歴を消す。
   - 消した後は履歴に現在位置 1 件だけを残す（戻る・進むは不可になる）。
   - localStorage の当該本の履歴キーも更新する（現在位置のみの履歴）。
   - 履歴が 1 件のときは disabled にする。
2. ARIA:
   - ボタンに `aria-label="Clear history"` を付ける。
3. 既存の挙動を変えない:
   - 既存の 119 テスト、E2E 116 項目、読書位置、履歴、表示設定、掃除、
     キーボード操作、タッチ操作、landmarks、page-list、履歴一覧、
     エクスポート・インポートを変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - 章を 2 回移動して履歴 3 件の状態でクリアを押すと、履歴が 1 件になり、
    Back ボタンが disabled になる。
  - クリア後も表示中の章は変わらない。
- 既存の 119 テストと E2E 116 項目が通る。

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
