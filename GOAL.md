# Goal

表紙 media-type の契約を回帰 test で固定する（R6 R1-archive#5）。

## Scope

### 背景

`is_image_media_type()` は ADR-0008 に従い、
未宣言（空）を許容し、明示された `image/` 以外を拒否する。
既存 test は後者と `image/png` 成功をカバーするが、
「未宣言を許容する」回帰 test が無かった。

### 成果物

- `core/src/epub.rs`: 変更（`is_image_media_type` の unit test）
- `docs/decisions/0249-cover-media-type.md`: 追加
- `todo.md`: R1-archive#5 完了、#2 / #3 は残存
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. `is_image_media_type("")` が true を返す test を追加する。
2. `image/png` / `IMAGE/SVG+XML` が true、`text/html` が false を固定する。
3. 本番コードは変更しない（test 追加のみ）。
4. 全ゲート成功（E2E 255 維持）。

## Acceptance criteria

- core test 増加、E2E 255 項目、全ゲート成功。

## Constraints

- 空 media-type を許容する ADR-0008 の決定は変更しない。
- 「空許容を継続するか」の再評価は未決定として残す（ADR-0240 を集約）。
- `master` へ push する。force push はしない。
