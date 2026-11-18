# Goal

Web リーダーにオプトインの縦書き表示を追加する（ADR-0145 の第二段、ADR-0146 のヘルパーを利用）。

## Scope

### 背景

ADR-0146 で E2E のスクロール読み書きを軸非依存化し、横書きで 10 回連続 229/0 を確認した。
次は Web 側の `ReaderSettings` に `writingMode` を追加し、
既定は現状の横書きのまま、オプトインで縦書きを試せるようにする。

### 成果物

- `web/src/lib.rs`: 変更（`writingMode` 設定、ツグル、スクロール/フラグメントの軸対応）
- `web/index.html`: 変更（縦書き用 CSS、ツグルボタンのスタイルは既存流用）
- `tools/verify-web-reader.py`: 変更（必要ならデバッグ情報のみ。期待総数 229 は維持）
- `docs/decisions/0147-opt-in-vertical-writing.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `ReaderSettings` に `writing_mode`（既定 `"horizontal-tb"`）を `serde(default)` で追加する。
2. リーダーツールバーに `#reader-writing-mode` ボタンを追加し、
   横書き／縦書き（`vertical-rl`）を切り替えられるようにする（既定は横書き）。
3. `.reader-container` に `vertical-writing` クラスを付け、
   CSS で `.reader-content` の `writing-mode` と overflow、
   `.reader-html img` の論理プロパティを切り替える。
4. アプリ側のスクロール読み書き・先頭リセット・フラグメント移動を
   `writing-mode` に応じて `scrollTop` / `scrollLeft` と `rect.top` / `rect.left` を選ぶようにする。
5. 保存 JSON のキーは 4 つ（`chapterIndex` / `fragment` / `savedAt` / `scrollTop`）を維持し、
   `scrollTop` には軸非依存オフセットを格納する（`scrollLeft` キーは追加しない）。
6. 横書きでは既存挙動と完全に同一であること。E2E 229/0、5 回連続 0 失敗。
7. 縦書きモードを CDP で手動検証し、`writing-mode: vertical-rl` と
   スクロール位置の保存・復元が動くことを確認する（checker 件数は増やさない）。
8. 全ゲートが成功する。

## Acceptance criteria

- 横書き E2E 229 項目 5 回連続 0 失敗。
- 縦書き手動検証で表示と位置保存／復元を確認。
- 6 ゲート + audit + wasm-test が成功する。

## Constraints

- 新規依存（Rust / npm / Python）を追加しない。
- 保存スキーマのキー名・個数は変更しない。
- ページ送り（改ページ）は今回行わない（ADR-0145 の後段）。
- `master` へ push する。force push はしない。
