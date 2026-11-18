# Goal

table の数値属性に fail-closed 検証を追加する（R6 PR-M02）。

## Scope

### 背景

`img` の `width` / `height` は `is_valid_dimension()` で検証されるが、
`td` / `th` の `colspan` / `rowspan`、`ol` の `start`、
`li` の `value`、`col` / `colgroup` の `span` は任意文字列を通す。
極端な値でレイアウト計算コストやメモリが増える可能性がある。

### 成果物

- `core/src/sanitize.rs`: 変更（数値属性検証と unit test）
- `docs/decisions/0187-table-attribute-validation.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. `colspan` / `rowspan` / `start` / `value` / `span` を
   ASCII 数字・1..=上限（推奨 1000）・桁数上限で検証する。
2. 不正値は属性ごと削除する（fail-closed、本文自体は残す）。
3. unit test を追加（正常値維持、0 / 負数 / 全角 / 上限超 / 長すぎる文字列の削除）。
4. 全ゲート成功（E2E 249 維持）。

## Acceptance criteria

- 全ゲート成功、E2E 249 項目成功。

## Constraints

- 新規依存を追加しない。
- `img` の寸法検証（1..=16384）は維持し、上限を共有しない。
- `master` へ push する。force push はしない。
