# Goal

Reader のアクセシビリティ軽微問題を修正する（R6 PR-L01 / PR-L02）。

## Scope

### 背景

- `.reader-content` に `aria-label="Chapter content"` と
  `aria-labelledby="reader-chapter-title"` が併記されており、
  ARIA では `aria-labelledby` が優先されるため `aria-label` は無効。
- ダーク切替ボタン `#reader-theme` に `aria-pressed` が無く、
  隣接する Auto / Downscale トグルと状態通知の挙動が不一致。

### 成果物

- `web/src/lib.rs`: 変更（aria-label 削除、aria-pressed 追加）
- `tools/verify-web-reader.py`: 変更（E2E 1 件、250 → 251）
- `docs/decisions/0198-reader-a11y-fixes.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. `.reader-content` から `aria-label="Chapter content"` を削除し、
   `aria-labelledby="reader-chapter-title"` に一本化する。
2. `#reader-theme` に `aria-pressed` を追加し、
   `dark.get()` の状態（true/false）を文字列で反映する。
3. 既存の title / aria-label / 表示文言 / ショートカットは維持する。
4. E2E 1 件追加（aria-label が無く aria-labelledby が有ること、
   aria-pressed が dark 状態と一致すること）。期待総数 250 → 251。
5. 全ゲート成功。

## Acceptance criteria

- E2E 251 項目、全ゲート成功。

## Constraints

- 新規依存を追加しない。
- 表示・操作挙動は変更しない（ARIA 属性のみ）。
- `master` へ push する。force push はしない。
