# Goal

dark テーマのボタン文字色を WCAG AA 水準へ修正する（R6 B07、ADR-0230）。

## Scope

### 成果物

- `web/index.html`: --color-on-accent を追加し .btn-primary のみ適用
- `docs/decisions/0230-dark-button-contrast.md`: 追加
- `todo.md`: B07 のコントラスト分を完了へ移動

### 決定

- dark primary は暗色文字（通常 4.85:1 / hover 7.02:1）。
- danger は白文字維持（4.83:1）。
- light 外観は不変、E2E 254 維持。

## Acceptance criteria

- 静的コントラスト比 AA 達成、E2E 254 項目・全ゲート成功（検証済み）。
