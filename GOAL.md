# Goal

章内の画像読み込みでレイアウトが変わっても、復元したスクロール位置が同じ
内容を指すようにする。

## Scope

### 背景

ADR-0016・0019 までで章内のスクロール位置を保存・復元するようにしたが、
画像は Blob URL で非同期に読み込まれるため、復元時点では画像の高さが未確定
のことがある。画像が後から入ると本文が押し下げられ、保存した `scrollTop` が
別の内容を指す（ADR-0016・ADR-0019 の未決定）。

### 成果物

- `web/src/lib.rs`: 変更（位置適用前の画像読み込み待ち）
- `tools/verify-web-reader.py`: 変更（レイアウトずれの E2E 検証を追加）
- `docs/decisions/0020-layout-shift-correction.md`: 追加（決定と検証の記録）
- `GOAL.md`: 本契約

### 必須事項

1. 位置の適用条件:
   - 本文（`.reader-html`）内で `src` がある `img` がすべて読み込み完了
     （`complete` または自然サイズ確定）するまで、スクロール位置の適用を
     待つ。
   - 待ち時間の上限は 2000ms とし、超えたら確定したレイアウトで適用する。
   - フラグメント指定と `scrollTop` 指定の両方で待つ。
   - 読み込みに失敗した画像（`complete` かつ自然サイズ 0）は待たない。
2. 再適用:
   - 待機後も要求した `scrollTop` に届かない場合は、従来どおり数フレーム
     再試行する。
   - 適用後の実際の位置を保存する（従来どおり）。
3. 既存の挙動を変えない:
   - 既存の 119 テスト、E2E 67 項目、復元、履歴、表示設定、掃除を変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - サンプル EPUB の第 2 章に縦長画像（600x900）を置き、画像の後ろにある
    見出し `#part` を基準にスクロール位置を保存する。
  - リロード後に復元したとき、`#part` の本文上端からの距離が保存時と
    一致する（許容 30px 以内）。
  - 画像が読み込み済みでも通常のスクロール復元が動く。
- 既存の 119 テストと E2E 67 項目が通る。

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

- 新規依存を追加しない。既存の `web-sys` 機能の範囲で実装する。
- 外部送信を追加しない。
- `desktop/` を変更しない。
- プロダクションコードで `unwrap()` / `expect()` / `panic!` / `todo!` を
  使わない。`unsafe` を使わない。
- コメントと doc コメントは日本語で簡潔に書く。
- 無関係なリファクタリングをしない。
- `master` へ push する。force push はしない。
