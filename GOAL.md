# Goal

TD-M10（壊れた画像による章ごとの 2 秒セトリング待ちの解消）は完了した。

## Scope

### 背景

`images_settled()` は `src` を持つ全画像に
`complete() && natural_width() > 0` を要求していた。
読み込み失敗画像は `complete()` が true でも自然幅 0 のままなので、
`IMAGE_SETTLE_MS = 2000` の deadline まで位置復元が待たされていた。
ADR-0225 で error を記録し失敗画像を待ち対象外にして解消した。

### 成果物（すべて反映済み）

- `web/src/lib.rs`: 画像の error を記録し、失敗画像を待ち対象外に。
- `tools/verify-web-reader.py`: E2E 1 件追加、253 → 254（check 237 + equal 17）。
- `docs/decisions/0225-broken-image-settling.md`: 追加（status: accepted）。
- `docs/e2e-traceability.md`: 254 件へ追従。新規 check を VC-004 の読書位置行へ割り当て。
- `GOAL.md` / `todo.md` / `README.md` / `docs/index.md` /
  `docs/decisions/index.md` / `docs/log.md`: 最終契約・完了反映。

### 必須事項（充足）

1. `resolve_images()` が生成した `<img>` の error を generation / chapter で排他して
   記録し、失敗画像を `images_settled()` の待ち対象から除外する — 完了。
2. 正常画像は従来どおり自然幅確定まで待ち、CLS / 位置復元契約を維持する — 完了。
3. 章切り替え・generation 変更時に古い画像のリスナーが誤って効かないようにする — 完了。
4. E2E 1 件を追加し（壊れた画像を含む章で位置復元が 1.5 秒未満で完了）、
   期待総数 253 → 254 とする — 完了。
5. 全ゲート成功 — ローカル検証で充足（py_compile / wasm-web check / verify.sh /
   E2E 254 すべて成功。CI は billing issue で実行不可）。

## Acceptance criteria（充足）

- E2E 254 項目すべて成功（check 237 + equal 17 = 254、期待総数 254 と一致）。
- AST 抽出: expected=254 / found=254 / distinct=254 / missing=0 / extra=0。
- 新規 E2E「壊れた画像を含む章の位置復元が 1.5 秒未満で完了する」が ok。
- 正常画像の自然幅待ち・CLS 契約・generation ガードを維持。
- markdownlint-cli2（`/tmp/mdlint.jsonc`）: 全対象 0 issues。
- 検証実績は ADR-0225 の Runtime Proof を出典とする。

## Constraints（遵守）

- 既存の EPUB 画像サニタイズ / Blob URL 寿命管理を変更しない。
- 新規依存を追加しない。unsafe なし。
- git 操作（commit / push）は本ドキュメント作業の範囲外とし、行わない。
