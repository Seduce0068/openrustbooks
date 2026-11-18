# Goal

E2E のスクロール読み書きを軸非依存ヘルパーへリファクタする（ADR-0145 の最初のステップ）。

## Scope

### 背景

ADR-0145 は、縦書き導入前に E2E のスクロール読み書きを
`writing-mode` に応じて `scrollTop` / `scrollLeft` を切り替える
ヘルパーへ集約し、挙動を変えずに横書きでベースラインを取り直すことを
最初のステップとした。

### 成果物

- `tools/verify-web-reader.py`: 変更（軸非依存ヘルパーと呼び出し置換）
- `docs/decisions/0146-axis-independent-e2e-scroll.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `.reader-content` のスクロール読み取り・設定・先頭リセットを
   共通ヘルパー（JS 式の生成関数等）へ集約する。
   - computed `writing-mode` が `vertical-*` なら `scrollLeft`、
     それ以外は `scrollTop` を使う（横Writing時は現状と同一）。
2. 位置保存の JSON キーは 4 つ（`chapterIndex` / `fragment` / `savedAt` / `scrollTop`）を維持。
   `scrollLeft` は追加しない。
3. E2E 期待総数 229、チェック名、閾値（2px / 50px / 30px）は変更しない。
4. スクロール順序計測（injected JS）も軸非依存にするが、
   横書きでは `scrollTop` を読む既存挙動を維持。
5. `python3 -m py_compile`、E2E 1 回、`./scripts/verify.sh` が成功する。
6. 別途 10 回連続実行でベースラインを確認する（親が実施）。

## Acceptance criteria

- 横書きで E2E 229/0、10 回連続 0 失敗。
- 6 ゲート + audit + wasm-test が成功する。

## Constraints

- アプリ本体（web/src/lib.rs 等）は変更しない。
- 新規依存を追加しない（Python 標準のみ）。
- `master` へ push する。force push はしない。
