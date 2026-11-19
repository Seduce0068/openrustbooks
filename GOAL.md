# Goal

`EpubBook::open_owned_with_limits()` を追加する（R6 R2-epub#1）。

## Scope

### 背景

`ArchiveReader` は limits 付きの所有 API `from_owned(bytes, limits)` を持つが、
`EpubBook::open_owned(bytes)` は既定 limits 固定だった。
将来 limits を要する呼び出しが借用 API `with_limits(&bytes, ...)` へ逃げると、
最大 256 MiB のバイト列を二重コピーする。
ADR-0051 の委譲パターンを所有 API にも適用する。

### 成果物

- `core/src/epub.rs`: 変更（`open_owned_with_limits()` 追加、`open_owned()` からの委譲、test 2 件）
- `docs/decisions/0247-epub-owned-limits.md`: 追加
- `todo.md`: R2-epub#1 完了、#2-#4 は未決定へ
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. `open_owned_with_limits(bytes: Vec<u8>, limits: ArchiveLimits) -> CoreResult<Self>` を追加する。
2. `open_owned()` は同関数へ `ArchiveLimits::default()` を渡して委譲する。
3. limits が所有経路でも効く unit test と、既定 limits で旧 API と同一になる test を追加する。
4. エラー文言にアーカイブ内パス・本文を出さない（ADR-0212）。
5. 全ゲート成功（E2E 255 維持）。

## Acceptance criteria

- core test 増加、E2E 255 項目、全ゲート成功。

## Constraints

- wasm.rs / web / E2E は変更しない（ADR-0190 の未決定を維持）。
- ファイル分割・欠落エラー統一・Document 重複解消は別案件。
- `master` へ push する。force push はしない。
