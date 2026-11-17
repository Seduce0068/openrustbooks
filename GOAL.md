# Goal

縮小 JPEG の品質を表示設定から変更できるようにする。

## Scope

### 背景

ADR-0072 の未決定「JPEG 品質を利用者が設定で変更できるようにするか」。
現在は 0.85 固定だが、画質を優先したい利用者やファイルサイズを優先したい
利用者が調整できない。既存の閾値設定と同じパターンで設定化する。

### 成果物

- `web/src/lib.rs`: 変更（`jpegQuality` 設定、トグル入力、縮小処理での使用）
- `tools/verify-web-reader.py`: 変更（E2E 検証を追加）
- `docs/decisions/0073-jpeg-quality-setting.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. 設定:
   - `ReaderSettings` に `jpegQuality`（`jpegQuality`）を追加。既定 85、
     範囲 50〜100（0.50〜1.00 を整数パーセントで保持）。
   - 既存設定にキーが無い場合は既定 85（後方互換）。保存・エクスポート・インポート対象。
   - ツールバーに数値入力（`<input id="reader-jpeg-quality">`）を置く。
2. 動作:
   - `downscale_large_images` で JPEG の encoder options に
     `quality: jpegQuality / 100.0` を使う。
3. 既存の挙動を変えない:
   - 縮小機能は既定オフ（ADR-0067）のまま。
   - 既存の 125 テスト、E2E 199 項目を変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - JPEG 品質入力が存在し、変更すると設定に保存される。
  - 範囲外はクランプされる。

## Acceptance criteria

- 6 ゲートすべて成功する。
- CI は billing issue で実行不可（ローカル検証で代替）。

## Constraints

- 新規依存を追加しない。
- `desktop/` を変更しない。
- プロダクションコードで `unwrap()` / `expect()` / `panic!` / `todo!` を
  使わない。`unsafe` を使わない。
- コメントと doc コメントは日本語で簡潔に書く。
- `master` へ push する。force push はしない。
