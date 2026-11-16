# Goal

巨大画像を Canvas で縮小するオプトイン設定を追加する（既定オフ）。

## Scope

### 背景

ADR-0064 で巨大画像の警告を追加したが、メモリ使用量そのものは未解消。
画像の自動縮小は既存の E2E（naturalHeight === 4000）やスクロール位置補正に
影響するリスクがあるため、**既定オフのオプトイン**として追加し、
利用者が選んだ場合だけ縮小する。

### 成果物

- `web/Cargo.toml`: 変更（web-sys の Canvas 系 feature 追加）
- `web/src/lib.rs`: 変更（設定・トグル・縮小処理）
- `tools/verify-web-reader.py`: 変更（E2E 検証を追加）
- `docs/decisions/0067-optional-image-downscale.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. 設定:
   - `ReaderSettings` に `downscaleImages`（`downscaleImages`）を追加。既定 false。
   - 既存設定にキーが無い場合は false（後方互換）。保存・エクスポート・インポート対象。
   - ツールバーにトグルボタン（`<button id="reader-downscale">`）を置く。
2. 縮小処理:
   - 有効時、本文画像のうち naturalWidth / naturalHeight が 4000px を超え、
     かつ naturalWidth が本文領域の幅より大きいものだけを対象にする
     （CSS の `max-width: 100%` で表示幅が決まる画像のみ。狭い画像を縮小すると
     レイアウトが変わるため除外する）。
   - 長辺が 2000px になる比率で Canvas に描き直し、Blob URL に差し替える。
   - 古い Blob URL は破棄し、新しい URL は既存の管理リストに追加して後で revoke する。
   - 縮小は表示サイズを変えないため、スクロール位置は再適用しない。
3. 既存の挙動を変えない:
   - 既定オフのため、既存の 125 テスト、E2E 189 項目は壊れないこと。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - 既定では缩小されない（既存サンプルで naturalHeight === 4000 のまま）。
  - トグルを有効にすると、4000px 超の画像が 2000px 以下に縮小される。
  - 狭い画像（幅が本文より小さい）は縮小されない。

## Acceptance criteria

- 6 ゲートすべて成功する。
- CI は billing issue で実行不可（ローカル検証で代替）。

## Constraints

- 新規クレートの依存を追加しない（既存 web-sys の feature 追加のみ）。
- `desktop/` を変更しない。
- プロダクションコードで `unwrap()` / `expect()` / `panic!` / `todo!` を
  使わない。`unsafe` を使わない。
- コメントと doc コメントは日本語で簡潔に書く。
- `master` へ push する。force push はしない。
