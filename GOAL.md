# Goal

Web 版 EPUB リーダーの本文画像に `decoding="async"` を付与し、
デコードを非同期化して本文の描画をブロックしない。

## Scope

### 背景

ADR-0020 までで画像の読み込み完了を待ってから位置を適用しているが、
画像のデコードはメインスレッドをブロックする。`loading="lazy"` は
`images_settled` が `complete` を待つため位置補正と相性が悪く、
これまで先送りされていた（ADR-0020 の未決定「画像の遅延読み込み
（`loading="lazy"`）と位置補正の併用」）。

### 成果物

- `web/src/lib.rs`: 変更（`resolve_images` で `decoding="async"` を付与）
- `tools/verify-web-reader.py`: 変更（E2E 検証を追加）
- `docs/decisions/0034-async-image-decoding.md`: 追加（決定と検証の記録）
- `GOAL.md`: 本契約

### 必須事項

1. 画像の非同期デコード:
   - `resolve_images` で `src` を Blob URL に差し替える各画像に
     `decoding="async"` を設定する。
   - `loading="lazy"` は付けない（`images_settled` が `complete` を
     待つため位置補正が壊れる。ADR-0020 の判断を維持する）。
2. 既存の挙動を変えない:
   - 既存の 119 テスト、E2E 129 項目、読書位置、画像の位置補正、
     履歴、表示設定、掃除、キーボード操作、タッチ操作、landmarks、
     page-list、履歴一覧、履歴クリア、エクスポート・インポート、
     マージ、カラースキーム追従を変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - 本文の画像に `decoding="async"` が付いている。
- 既存の 119 テストと E2E 129 項目が通る。

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
