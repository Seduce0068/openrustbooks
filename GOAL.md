# Goal

縮小時の JPEG 再エンコードに品質パラメータを指定する。

## Scope

### 背景

ADR-0071 の未決定「JPEG の再エンコード品質（quality）の最適値」。
`to_blob_with_type()` の既定品質はブラウザ依存で、ファイルサイズと画質の
バランスが安定しない。JPEG には 0.85 を明示し、再現性を確保する。

### 成果物

- `web/src/lib.rs`: 変更（`to_blob_with_type_and_encoder_options` の使用）
- `tools/verify-web-reader.py`: 変更（E2E 検証を追加）
- `docs/decisions/0072-downscale-jpeg-quality.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. 品質指定:
   - MIME が `image/jpeg` の場合、`{ quality: 0.85 }` をencoder options として
     `to_blob_with_type_and_encoder_options()` を呼ぶ。
   - PNG / WebP は従来どおり `to_blob_with_type()` を使う
     （quality は形式により意味が異なるため指定しない）。
   - 既定オフ（ADR-0067）や表示サイズ不変の方針は維持。
2. 既存の挙動を変えない:
   - 既存の 125 テスト、E2E 199 項目を変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - JPEG 画像を縮小しても MIME が `image/jpeg` のままである。

## Acceptance criteria

- 6 ゲートすべて成功する。
- CI は billing issue で実行不可（ローカル検証で代替）。

## Constraints

- 新規依存を追加しない。
- `desktop/` を変更しない。
- コメントと doc コメントは日本語で簡潔に書く。
- `master` へ push する。force push はしない。
