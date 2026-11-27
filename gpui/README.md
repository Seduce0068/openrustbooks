# openrustbooks GPUI viewer (prototype)

openrustbooks-core を再利用した、Rust ネイティブな GPUI 版ビューアのプロトタイプです。
既存の Tauri/Leptos 版・Android 版・PWA は変更しません。設計判断は
[ADR-0354](../docs/decisions/0354-gpui-viewer-prototype.md) を参照してください。

## なぜ独立 workspace か

ルート workspace は MSRV 1.95 と wasm/Android のクロスビルドを契約としています。
GPUI はネイティブ图形スタックと MSRV 1.100 を要求するため、このクレートは
ルートの members に入れず、専用の Cargo.lock を持ちます。

## 依存

- `gpui-pre` / `gpui-pre-platform` 0.3.12（Zed スナップショット、公式 gpui 0.2.2 は Linux でビルド失敗）
- `openrustbooks-core`（path 依存、EPUB/ZIP 解析・サニタイズ・設定型を再利用）

## 起動

```bash
cd gpui
cargo build --locked
cargo run --locked -- /path/to/book.epub
# または OPENRUSTBOOKS_EPUB=/path/to/book.epub cargo run --locked
```

画面には書誌情報・章タイトル・本文（プレーンテキスト）と、前の章 / 次の章ボタンを表示します。

## 検証

`gpui-pre` / `gpui-pre-platform` の `test-support` feature を dev-dependency として
有効化すると、`#[gpui::test]` と `TestAppContext` でウィンドウを生成し、
`Window::painted_quads()` からラスタライズ前の描画クワッドを検証できます。
OS ウィンドウを表示せずに、ナビゲーションボタンの配色とパス未指定時の
ステータスを確認するテストを `reader.rs` に追加しています。
`book.rs` の既存テストは変更していません。

```bash
cd gpui
cargo fmt --all
cargo check --locked
cargo test --locked
cargo test --locked reader_view_paints_expected_backgrounds
cargo test --locked reader_view_without_path_has_fixed_status
cargo clippy --all-targets --locked -- -D warnings -D clippy::undocumented_unsafe_blocks
```

### cargo test 結果

`cargo test`: rc=0、**5 passed / 0 failed**（book.rs の純粋ロジック 3 件 +
reader.rs の GPUI テスト 2 件）。`cargo fmt --all` / `cargo check --all-targets` /
`cargo clippy --all-targets -- -D warnings -D clippy::undocumented_unsafe_blocks` も rc=0。

OS レベル検証では、`ydotool` で「次の章」ボタンをクリックし、章表示が `1 / 2` から
`2 / 2` へ変化することを `spectacle` + `tesseract` で確認済み（ADR-0354 参照）。

## 未実装

画像・表紙・スクロール位置保持・設定 UI・AI 送信・KeyBinding は未実装です。
機能完全版としては既存の Tauri/Leptos 版を使用してください。
