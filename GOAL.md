# Goal

要件の検証条件と E2E check 名の対応表を追加する（R6 TD-L07、ADR-0206）。

## Scope

### 成果物

- `docs/requirements.md`: VC-001〜015
- `docs/e2e-traceability.md`: 251 check 名の対応表
- `docs/decisions/0206-e2e-traceability.md`: 追加
- `todo.md`: TD-L05 / TD-L07 を完了へ移動

### 決定

- check 名は AST 抽出の実名のみ、251 件完全一致。
- 未カバー要件は理由付きで明記。
- 期待総数はコード出典（ADR-0090）。

## Acceptance criteria

- E2E 251 項目、全ゲート成功（検証済み）。
