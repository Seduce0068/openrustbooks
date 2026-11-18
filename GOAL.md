# Goal

画像縮小 E2E のフレークを修正する（ADR-0140 の未決定）。

## Scope

### 背景

10 回連続実行の第 3 回で
`有効時は巨大画像が 2000px 以下に縮小される`（naturalWidth=4500）が失敗した。
`wait_until(..., timeout=10)` の戻り値を無視しており、
縮小完了待ちがタイムアウトしても再待機せずに判定している（**推測**）。

### 成果物

- `tools/verify-web-reader.py`: 変更（待機ロジックの安定化）
- `web/src/lib.rs`: 変更（必要なら縮小完了の通知）
- `docs/decisions/0141-downscale-flake.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. 失敗原因をコードから特定する（待機不足／縮小処理の競合／-generation guard 等）。
2. 原因に応じて修正する。E2E の待機を条件待ち＋再確認にするか、
   縮小完了を明示的に待つ方法を選ぶ。
3. E2E 期待総数 229 は維持する。
4. 5 回連続実行で 0 失敗を確認する。
5. 全ゲートが成功する。

## Acceptance criteria

- E2E 229 項目、5 回連続 0 失敗。
- 6 ゲート + audit + wasm-test が成功する。

## Constraints

- 縮小ポリシー（4000px 超・2000px 以下・MIME 保持）は変更しない。
- 新規依存を追加しない。
- `master` へ push する。force push はしない。
