# Goal

画像縮小時に元画像の MIME タイプを保持する。

## Scope

### 背景

ADR-0067 の未決定「縮小後の画質比較と、JPEG 再エンコードの選択」。
現在は `canvas.to_blob()` の既定（PNG）で再エンコードしているため、
JPEG の写真画像でも PNG になり、ファイルサイズが増えることがある。
また、GIF のアニメーション情報は失われる（縮小自体が静止化するため
既知の制約）。元の MIME を保持し、JPEG/PNG/WebP は同じ形式で再エンコードする。

### 成果物

- `web/src/lib.rs`: 変更（元 Blob の MIME 取得と `to_blob_with_type` 使用）
- `tools/verify-web-reader.py`: 変更（E2E 検証を追加）
- `docs/decisions/0071-downscale-mime-preservation.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. MIME 保持:
   - 縮小前に元画像の `src`（Blob URL）を `fetch` して Blob の `type` を取得する。
   - 取得した MIME が `image/jpeg` / `image/png` / `image/webp` のいずれかなら
     `canvas.to_blob_with_type()` で同じ形式に再エンコードする。
   - それ以外（GIF 等）は従来どおり既定（PNG）を使う。
   - MIME 取得に失敗した場合も既定（PNG）にフォールバックする（fail-safe）。
2. 既存の挙動を変えない:
   - 縮小機能は既定オフ（ADR-0067）のまま。
   - 既存の 125 テスト、E2E 198 項目を変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - 縮小後の画像 Blob の MIME が元画像と同じである（JPEG の場合）。

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
