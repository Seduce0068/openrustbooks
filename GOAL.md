# Goal

EPUB 本編を責務別モジュールへ分割する（R6 R2-epub#2）。

## Scope

### 背景

ADR-0255 で tests 1897 行を `epub/tests.rs` へ切り出し、
`core/src/epub.rs` は本編 1880 行になった。
残る本編には container/OPF、nav/NCX、XHTML テキスト抽出、
共通 XML/パス helper が混在しており、
公開 6 型と fail-closed 契約を維持したまま子モジュールへ分離する。

### 成果物

- `core/src/epub.rs`: 変更（公開 6 型 + `impl EpubBook` + 子モジュール宣言 / re-export）
- `core/src/epub/opf.rs`: 追加（container / OPF 解析、private データ構造）
- `core/src/epub/navigation.rs`: 追加（nav / NCX 解析）
- `core/src/epub/text.rs`: 追加（XHTML テキスト / タイトル抽出）
- `core/src/epub/xmlutil.rs`: 追加（属性・パス・正規化の共通 helper）
- `docs/decisions/0260-epub-module-split.md`: 追加
- `todo.md`: R2-epub#2 完了
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. `lib.rs` が re-export する公開 6 型（ChapterLink / EpubBook / EpubChapter /
   EpubCoverImage / EpubMetadata / EpubTocEntry）を同じ Rust パスで維持する。
2. `EpubBook` の公開メソッド挙動、fail-closed なパス / スキーム検証、
   media-type 契約、上限値を変更しない。
3. `epub/tests.rs` の本文は変更しない（必要なら `pub(super)` で親へ見せる）。
4. core test 284 / wasm 14 / E2E 255 を維持する。
5. 全ゲート成功。

## Acceptance criteria

- core 284 / wasm 14 / E2E 255、全ゲート成功。

## Constraints

- test 本文・E2E・Cargo.toml / Cargo.lock・公開 API は変更しない。
- `unsafe` を追加しない。
- `master` へ push する。force push はしない。
