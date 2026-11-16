# Goal

本文の巨大画像を検出し、読み込み後に状態メッセージで警告する。

## Scope

### 背景

ADR-0034 の未決定「画像のデコード・縮小（大きな画像のメモリ使用）」。
本文画像は `max-width: 100%` で表示縮小されるが、メモリ上は元のピクセル
（例: 4000px）が保持される。いきなり縮小すると既存 E2E
（naturalHeight === 4000）やスクロール位置補正に影響するため、
まず巨大画像を検出して利用者に知らせる。

### 成果物

- `web/src/lib.rs`: 変更（巨大画像の検出と警告）
- `web/index.html`: 変更（警告のスタイル、必要に応じて）
- `tools/verify-web-reader.py`: 変更（E2E 検証を追加）
- `docs/decisions/0064-large-image-warning.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. 検出と警告:
   - 章表示後、本文の `<img>` の naturalWidth / naturalHeight を調べ、
     いずれかが 4000px を超える画像を検出する。
   - 検出した場合、状態メッセージに「Large image detected (WxH). Consider a smaller source.」を表示する。
   - 警告は表示のみで、画像の差し替え・スクロール位置の変更は行わない。
2. 既存の挙動を変えない:
   - 既存の 124 テスト、E2E 185 項目（naturalHeight === 4000 の検証を含む）を変えない。
   - 閾値ちょうど（4000px）は警告対象にしない（既存サンプルを壊さない）。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - 既存サンプル（4000px）では警告が出ない。
  - 閾値を超える画像（E2E 内で生成）で警告が出る。

## Acceptance criteria

- 6 ゲートすべて成功する。
- CI は billing issue で実行不可（ローカル検証で代替）。

## Constraints

- 新規依存を追加しない。
- `desktop/` を変更しない。
- コメントと doc コメントは日本語で簡潔に書く。
- `master` へ push する。force push はしない。
