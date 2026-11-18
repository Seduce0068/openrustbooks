# Goal

画像縮小の完了シグナルを追加して E2E の再フレークを解消する（ADR-0141 の残り）。

## Scope

### 背景

ADR-0141 で `wait_until` の戻り値確認と 10 秒再待機を入れたが、
縦書き実装後の 5 回連続で 2 回 `有効時は巨大画像が 2000px 以下に縮小される`
（naturalWidth=4500）が失敗した。完了シグナルが無いため、
縮小が走らなかった場合と遅れた場合を区別できない。

### 成果物

- `web/src/lib.rs`: 変更（縮小完了時に `data-downscaled="1"` を付与）
- `tools/verify-web-reader.py`: 変更（完了シグナル待ち＋診断、期待総数 229 維持）
- `docs/decisions/0148-downscale-completion-signal.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `downscale_large_images()` が `img.src` を差し替えてデコード完了した時点で、
   対象 `img` に `data-downscaled="1"` を付与する。
2. E2E は `naturalWidth <= 2000` に加えて
   `img[data-downscaled="1"]` を待つ（チェック名・期待総数 229 は維持）。
3. 縮小ポリシー（4000px 超、最小長辺 max(2000, content_width)、MIME 保持）は変更しない。
4. 10 回連続 E2E で 0 失敗を確認する。
5. 全ゲートが成功する。

## Acceptance criteria

- E2E 229 項目、10 回連続 0 失敗。
- 6 ゲート + audit + wasm-test が成功する。

## Constraints

- 新規依存を追加しない。
- 既存 E2E チェック名・件数・閾値は変更しない（待ち条件のみ）。
- `master` へ push する。force push はしない。
