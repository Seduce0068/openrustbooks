# Goal

Web 版 EPUB リーダーで、章の移動履歴を localStorage に保存し、同じ本を
開き直したときにも戻る・進むができるようにする。

## Scope

### 背景

ADR-0017 までで移動履歴はメモリ上にのみ保持され、本を開き直すと履歴が
消えていた（ADR-0017 の未決定「履歴の永続化（再起動後も戻れるようにするか）」）。
読書位置はすでに localStorage に保存しているため、履歴も同じキー体系で
保存する。

### 成果物

- `web/src/lib.rs`: 変更（履歴の保存・復元）
- `tools/verify-web-reader.py`: 変更（履歴永続化の E2E 検証を追加）
- `docs/decisions/0024-history-persistence.md`: 追加（決定と検証の記録）
- `GOAL.md`: 本契約

### 必須事項

1. 保存:
   - 履歴のキーは `openrustbooks:history:<name>:<size>:<lastModified>`。
   - 値は JSON `{ "entries": [{"chapterIndex": N, "fragment": "..." | null,
     "scrollTop": N}], "index": N }`。
   - 章を移動するたびに保存する（履歴が変わるたび）。
   - 履歴が空・1件のときは保存しない（初期状態）。
   - localStorage が使えない場合は黙って諦める。
2. 復元:
   - 本を開いたとき、保存された履歴があり、すべての `chapterIndex` が章数の
     範囲内なら復元する。
   - 範囲外のエントリが含まれる場合は履歴を捨て、初期状態にする。
   - 保存が無い場合は従来どおり初期状態。
   - 復元後は Back / Forward ボタンの有効状態も復元する。
3. 寿命・掃除:
   - 履歴は読書位置の掃除（90 日・40 件）の対象にしない。
   - 本を開き直すたびに上書きする。
4. 既存の挙動を変えない:
   - 既存の 119 テスト、E2E 97 項目、読書位置、表示設定、掃除を変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - 章を移動すると履歴が localStorage に保存される。
  - リロードして同じ本を開くと Back が有効になる。
  - リロード後に Back で前の章へ戻れる。
  - 範囲外の履歴値では履歴がリセットされる。
- 既存の 119 テストと E2E 97 項目が通る。

## Acceptance criteria

- 上記の必須事項がコードと E2E で確認できる。
- 次がすべて成功する。
  - `cargo fmt --all ----check`
  - `cargo check --all-targets --locked`
  - `cargo test --all-targets --locked`
  - `cargo clippy --all-targets --locked -- -D warnings`
  - `cargo check -p openrustbooks-core --target wasm32-unknown-unknown --locked`
  - `cargo check -p openrustbooks-web --target wasm32-unknown-unknown --locked`
  - `trunk build --offline --dist <一時出力先>`（web）
  - `python3 tools/verify-web-reader.py --dist <出力先>`（E2E）
- CI（`Seduce0068/workflows/rust-gates.yml@v1`）が成功する。

## Constraints

- 新規依存を追加しない。保存先は localStorage のみ。外部送信を追加しない。
- `desktop/` を変更しない。
- プロダクションコードで `unwrap()` / `expect()` / `panic!` / `todo!` を
  使わない。`unsafe` を使わない。
- コメントと doc コメントは日本語で簡潔に書く。
- 無関係なリファクタリングをしない。
- `master` へ push する。force push はしない。
