# Goal

dark テーマのボタン文字色を WCAG AA 水準へ修正する（R6 B07 のコントラスト）。

## Scope

### 背景

監査で、light の muted / dark の muted は AA を満たす一方、
dark の `.btn-primary` は白文字 / `#3b82f6` で **3.68:1**、
hover の `#60a5fa` では **2.54:1** となり WCAG 1.4.3（4.5:1）未達と判定された。
`.btn-danger` も白文字で同型のリスクがある。

### 成果物

- `web/index.html`: 変更（dark 用の `--color-on-accent` 追加、ボタン文字色を変数化）
- `docs/decisions/0230-dark-button-contrast.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. dark テーマでプライマリ／デンジャーボタンの文字色を暗色へ変更する。
2. hover 時も 4.5:1 以上を満たす（静的計算で確認）。
3. light テーマの外観は変更しない。
4. E2E 254 を維持する（色値への直接依存なしを確認）。
5. 全ゲート成功。

## Acceptance criteria

- 静的コントラスト比が dark 通常/hover ともに 4.5:1 以上、E2E 254 維持。

## Constraints

- 新規依存を追加しない。
- レイアウト・操作挙動は変更しない。
- `master` へ push する。force push はしない。
