# Goal

EPUB 構造要素の数上限を追加する（第2cycle R6 / R3-SE01）。

## Scope

### 背景

ZIP エントリ数とサイズには上限があるが、1 つの OPF / NCX 文書内の
`<item>` / `<itemref>` / `<meta>` キャプチャ / `<navPoint>` は無制限で、
64 MiB の文書から数百万要素を確保し得る。

### 成果物

- `core/src/epub/*`: 変更
- `docs/decisions/0316-epub-structure-limits.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. manifest / spine / captures / NCX navPoint の蓄積時点で上限を適用する。
2. 上限超過は `CoreError::Validation` で fail-closed に拒否する。
3. 既存 `MAX_TOC_ENTRIES=10_000` と整合した上限を使う。
4. 単体 test: 各要素の境界値と超過拒否、正常 EPUB の従来動作。
5. verify.sh 全ゲート成功、E2E 256 件・WDIO 維持。

## Constraints

- 新規依存 / unsafe FFI は使わない。
- 既存 EPUB / ZIP の fail-closed 契約と check 名を変えない。
- `master` へ push する。force push はしない。
