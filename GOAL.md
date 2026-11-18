# Goal

Web エディタを textarea ベースの最小構成へ配線する（ADR-0127 の第一段）。

## Scope

### 背景

`web/src/lib.rs` の Editor コンポーネントは `editor-pane` が空の div で、
Save は `saving` を切り替えるだけ、AI / Lint ボタンは未配線。
ADR-0127 は CodeMirror を導入せず textarea 最小構成で進める方針を決定済み。

### 成果物

- `web/src/lib.rs`: 変更（textarea 入力、Preview 接続、未実装ボタンの明示）
- `web/index.html`: 変更（textarea のスタイル最小追加）
- `tools/verify-web-reader.py`: 変更（E2E 2 件追加、235 → 237）
- `docs/decisions/0161-minimal-editor-ui.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `editor-pane` を `<textarea id="editor-input">` に置き換え、
   `content` シグナルと双方向バインドする。
2. Preview が入力をリアルタイムにサニタイズ表示する（既存 `Preview` を再利用）。
3. Save / AI Generate / AI Refine / Lint ボタンは、未接続を誤解させないよう
   disabled ＋ title/aria で「デスクトップ版のみ／未実装」と明示する。
4. 書式 select も disabled にする（機能未接続のため）。
5. E2E に 2 件追加:
   - `/editor/e2e` で textarea に Markdown を入力すると Preview に反映される。
   - 未実装ボタンが disabled である。
6. 既存 235 項目と全ゲートを壊さない。期待総数を 237 に更新。

## Acceptance criteria

- E2E 237 項目が成功する。
- 6 ゲート + audit + wasm-test が成功する。

## Constraints

- CodeMirror や npm 依存を追加しない。
- Tauri invoke は有効化しない（ADR-0128 の判断を維持）。
- 保存処理は実装しない（誤解を与えない表示に留める）。
- `master` へ push する。force push はしない。
