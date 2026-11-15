# Goal

Web 版 EPUB リーダーに移動履歴の一覧 UI を追加する。

## Scope

### 背景

ADR-0017・ADR-0024 までで移動履歴はメモリと localStorage に保持され、
戻る・進むボタンで移動できるが、過去の位置へ直接ジャンプする手段が無い
（ADR-0017 の未決定「履歴一覧の UI」）。

### 成果物

- `web/src/lib.rs`: 変更（履歴一覧のドロップダウンとジャンプ）
- `tools/verify-web-reader.py`: 変更（履歴一覧の E2E 検証を追加）
- `docs/decisions/0029-history-list-ui.md`: 追加（決定と検証の記録）
- `GOAL.md`: 本契約

### 必須事項

1. 履歴一覧:
   - ツールバーに履歴のドロップダウン（`<select id="reader-history">`）を置く。
   - 各エントリのラベルは「章のタイトル + フラグメント」（フラグメントが
     無い場合は章のタイトルだけ）。
   - 現在位置のエントリが選択状態になる。
   - 選択を変えると、その履歴エントリの章・フラグメント・スクロール位置へ
     移動する（履歴の現在位置も更新する）。
   - 履歴が 1 件以下のときは disabled にする。
2. ARIA:
   - ドロップダウンに `aria-label="History"` を付ける。
3. 既存の挙動を変えない:
   - 既存の 119 テスト、E2E 110 項目、読書位置、履歴、表示設定、掃除、
     キーボード操作、タッチ操作、landmarks、page-list を変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - 章を 2 回移動すると履歴一覧に 3 件入る。
  - 一覧から最初のエントリを選ぶと最初の章へ移動する。
- 既存の 119 テストと E2E 110 項目が通る。

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
