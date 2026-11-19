# Goal

EPUB テストを `epub/tests.rs` へ切り出す（R6 R2-epub#2 の第一段）。

## Scope

### 背景

`core/src/epub.rs` は実測 3780 行で、本編 1880 行 / tests 1900 行。
既存 test 92 件のうち private item 直接参照は
`is_image_media_type` と `MAX_TOC_ENTRIES` の 2 つだけで、
双方を親モジュールへ残せば `use super::*` を維持できる。
共有 fixture を壊さず、挙動を変えずにファイルだけ分割する。

### 成果物

- `core/src/epub.rs`: 変更（tests module を #[path] 参照へ置換）
- `core/src/epub/tests.rs`: 追加（既存 tests 本文を移動）
- `docs/decisions/0255-epub-test-split.md`: 追加
- `todo.md`: R2-epub#2 を「tests 分割完了、本編分割は未決定」へ更新
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. test 本文 1 行も変更せず `core/src/epub/tests.rs` へ移動する。
2. `core/src/epub.rs` の本編（公開 API / private helper / 定数）は変更しない。
3. `#[cfg(test)] #[path = "epub/tests.rs"] mod tests;`（non-mod-rs の path 規則）で参照し、
   92 tests がすべて検出されること。
4. 本編の 3/4 モジュール分割は ADR の未決定へ残す。
5. 全ゲート成功（E2E 255 維持）。

## Acceptance criteria

- core test 件数不変（284 件）、E2E 255、全ゲート成功。

## Constraints

- 公開 API / 可視性 / test 内容 / E2E は変更しない。
- `master` へ push する。force push はしない。

## 完了記録

- core/src/epub.rs を 3780→1884 行、tests.rs へ 1897 行・92 tests をバイト単位で移動。
- ADR-0255: 本編 3/4 分割、epub/mod.rs 化、integration test 化は未決定。
- `./scripts/verify.sh`: 全ゲート成功、E2E 255 passed / 0 failed。
