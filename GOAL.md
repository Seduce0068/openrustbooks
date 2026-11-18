# Goal

Reader の ARIA 属性を修正する（R6 PR-L01 / PR-L02、ADR-0198）。

## Scope

### 成果物

- `web/src/lib.rs`: aria-label 削除、aria-pressed/可視ラベルを effective_dark へ
- `tools/verify-web-reader.py`: E2E 1 件、250 → 251
- `docs/decisions/0198-reader-a11y-fixes.md`: 追加（ADR-0058 を supersede）
- `todo.md`: PR-L01 / L02 を完了へ移動

### 決定

- aria-labelledby に一本化。
- toggle 状態は実効値で告知。
- 空タイトル時の accessible name は未決定。

## Acceptance criteria

- E2E 251 項目、全ゲート成功（検証済み）。
