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

## キーボード操作

起動時にルートビューへフォーカスを設定します。章が 2 章以上ある場合、次のキーで移動できます。

| キー | 動作 |
| --- | --- |
| 右矢印 | 次の章へ移動（末尾ならそのまま） |
| 左矢印 | 前の章へ移動（先頭ならそのまま） |

章が 1 章のみの場合はキー操作では移動しません（既存ボタンと同じ can_navigate ガード）。
ボタンの onclick 挙動は変更していません。

## 検証

`gpui-pre` の `test-support` feature を dev-dependency として有効化すると、
`#[gpui::test]` と `TestAppContext` でウィンドウを生成し、
`Window::painted_quads()` からラスタライズ前の描画クワッドを検証できます。
`reader.rs` には描画色・パス未指定時のステータス・2 章 EPUB での左右キー移動
（0 → 1 → 1 でクランプ → 0）を検証するテストを追加しています。
`book.rs` の既存テストは変更していません。

```bash
cd gpui
cargo fmt --all
cargo check --all-targets --locked
cargo test --locked
cargo test --locked reader_view_paints_expected_backgrounds
cargo test --locked reader_view_without_path_has_fixed_status
cargo test --locked arrow_keys_move_chapter
cargo clippy --all-targets --locked -- -D warnings -D clippy::undocumented_unsafe_blocks
```

OS レベルでは、対象ウィンドウをアクティブにしたうえで左右キーを送り、
章表示が 1 / 2 → 2 / 2 → 1 / 2 と変化することを確認します。

```bash
# 対象ウィンドウを活性化（windowactivate に --sync は無いため ID を再確認する）
id="$(kdotool search --name 'openrustbooks' | tail -n 1)"
kdotool windowactivate "$id"
sleep 1
echo "active=$(kdotool getactivewindow) expected=$id"
kdotool getactivewindow getwindowname

# 右矢印（keycode 106）と左矢印（keycode 105）
ydotool key -d 50 106:1 106:0
ydotool key -d 50 105:1 105:0
```

### 検証結果

- `cargo fmt --all` / `cargo check --all-targets` /
  `cargo clippy --all-targets -- -D warnings -D clippy::undocumented_unsafe_blocks`: rc=0
- `cargo test`: rc=0、**6 passed / 0 failed**（`arrow_keys_move_chapter` を含む）
- OS レベル: 活性化した Window へ `ydotool key -d 50 106:1 106:0` で 1 / 2 → 2 / 2、
  `105:1 105:0` で 2 / 2 → 1 / 2 を `spectacle` + `tesseract` の OCR で確認。

## 未実装

画像・表紙・スクロール位置保持・設定 UI・AI 送信・設定ファイルによる KeyBinding は未実装です（左右キーの固定操作のみ実装）。
機能完全版としては既存の Tauri/Leptos 版を使用してください。
