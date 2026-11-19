# Goal

Document 側の dead な EPUB 型を削除する（R6 R2-epub#4）。

## Scope

### 背景

`core/src/document.rs` の `DocumentContent::Epub` と
`EpubSpineItem` / `EpubManifestItem` / `EpubTocItem` は
リポジトリ内参照がゼロで、EPUB 閲覧は `EpubBook` / `WasmEpubBook` の
別経路で提供されている。
`DocumentContent` 自体も `lib.rs` で re-export されていない。
ただし `pub mod document` 経由で公開到達可能で、
serde variant としては破壊的変更になる。

### 成果物

- `core/src/document.rs`: 変更（4 項目を削除）
- `docs/decisions/0250-document-epub-type-removal.md`: 追加
- `todo.md`: R2-epub#4 完了、#2 は残存
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. `DocumentContent::Epub` variant と 3 つの pub struct を削除する。
2. `DocumentContent` の他 variant、`epub.rs`、desktop / web / wasm は変更しない。
3. 外部 Rust / serde 利用者の有無は未決定として ADR に記録する。
4. 全ゲート成功（E2E 255 維持）。

## Acceptance criteria

- core test 成功、E2E 255 項目、全ゲート成功。

## Constraints

- `epub.rs` 型への一本化は行わない（変換対象ゼロのため別案件）。
- 新規依存を追加しない。
- `master` へ push する。force push はしない。
