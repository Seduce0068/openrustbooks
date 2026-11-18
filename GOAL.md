# Goal

table の数値属性に fail-closed 検証を追加する（R6 PR-M02、ADR-0187）。

## Scope

### 成果物

- `core/src/sanitize.rs`: MAX_TABLE_NUMBER / is_valid_table_number / attribute_filter 拡張、unit 4 件
- `docs/decisions/0187-table-attribute-validation.md`: 追加
- `todo.md`: PR-M02 を完了へ移動

### 決定

- colspan / rowspan / start / value / span を ASCII 数字 1..=1000 に制限。
- 不正値は属性のみ削除、本文は維持。
- img の 1..=16384 契約は維持。

## Acceptance criteria

- core 224 / E2E 249、全ゲート成功（検証済み）。
