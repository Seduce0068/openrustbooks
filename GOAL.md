# Goal

E2E Checker に check 名の一意性検証を追加する（R6 TD-M03、ADR-0182）。

## Scope

### 成果物

- `tools/verify-web-reader.py`: Checker に names 集合と重複検知を追加
- `docs/decisions/0182-e2e-unique-check-names.md`: 追加
- `todo.md`: TD-M03 を完了へ移動

### 決定

- 同名 check は失敗として記録し、passed を増やさない。
- equal() も同じ名前空間で検知。
- 既存 249 件はすべて一意で件数維持。

## Acceptance criteria

- E2E 249 項目、全ゲート成功（検証済み）。
