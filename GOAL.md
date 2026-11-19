# Goal

EPUB manifest の欠落エラーを統一した（R6 R2-epub#3、ADR-0248）。

## Scope

### 成果物

- `core/src/epub.rs`: id / href 欠落を固定文言の Archive エラー化、test 2 件
- `docs/decisions/0248-epub-manifest-errors.md`: 追加
- `todo.md`: R2-epub#3 完了、#2 / #4 残存
- README / 各索引 / log: 更新

### 決定

- id 欠落は黙ってスキップせず fail-closed。
- href 欠落から {id} を削除し、値非露出の固定文言へ統一。
- id="" と他エラー文言の redaction は未決定。

## Acceptance criteria

- core 278 / E2E 255、全ゲート成功（検証済み）。
