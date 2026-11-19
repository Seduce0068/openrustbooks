# Goal

`prefers-contrast: more` で枠線と補助文字のコントラストを補強する（R6 B06 / C01 残）。

## Scope

### 背景

`forced-colors` は ADR-0039 で対応済みだが、
`prefers-contrast: more` は「必要性なし」として未対応だった。
しかし static 計算で light の枠線 `#e5e7eb` / 白 = 1.24:1、
dark の枠線 `#334155` / `#0f172a` = 1.72:1 と
WCAG 1.4.11（非テキスト 3:1）に不足していた。

### 成果物

- `web/index.html`: 変更（`prefers-contrast: more` のトークン上書き）
- `docs/decisions/0236-prefers-contrast.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. `@media (prefers-contrast: more)` で light / dark の
   `--color-border` を 3:1 以上へ強化する。
2. `--color-muted` も補強し、通常テキストの 4.5:1 を維持する。
3. forced-colors の既存契約（ADR-0039）を壊さない。
4. E2E 254 を維持する（色値への直接依存なし）。
5. 全ゲート成功。

## Acceptance criteria

- 静的コントラスト比が枠線 3:1 以上、muted 4.5:1 以上、E2E 254 維持。

## Constraints

- 一般テーマの配色は変更しない（media query 内のみ）。
- 新規依存を追加しない。
- `master` へ push する。force push はしない。
