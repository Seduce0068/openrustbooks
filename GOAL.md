# Goal

`read_text()` の不正 UTF-8 内容漏洩を修正する（R6 PR-H02/TD-H02、ADR-0174）。

## Scope

### 成果物

- `core/src/io.rs`: `match String::from_utf8(bytes)` + U+FFFD 置換、テスト 2 件、doc 修正
- `docs/decisions/0174-read-text-utf8.md`: 追加
- `todo.md`: PR-H02/TD-H02 を完了へ移動

### 決定

- 有効 UTF-8 はゼロコピー、不正時のみ lossy 変換。
- どちらの経路でもエラーを生成せず、内容をエラー文言へ載せない。
- EPUB / ZIP の fail-closed は維持。

## Acceptance criteria

- E2E 246 項目、全ゲート成功（検証済み）。
