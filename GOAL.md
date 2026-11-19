# Goal

EPUB 所有 API へ limits を追加した（R6 R2-epub#1、ADR-0247）。

## Scope

### 成果物

- `core/src/epub.rs`: open_owned_with_limits 追加と open_owned の委譲、test 2 件
- `docs/decisions/0247-epub-owned-limits.md`: 追加
- `todo.md`: R2-epub#1 完了、#2-#4 は未決定
- README / 各索引 / log: 更新

### 決定

- ArchiveReader::from_owned(bytes, limits) へ委譲。
- wasm / web / E2E は変更せず、既定 limits は暫定値として維持。

## Acceptance criteria

- core 276 / wasm 15 / E2E 255、全ゲート成功（検証済み）。
