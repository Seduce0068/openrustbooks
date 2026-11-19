# Goal

status 文言を日本語へ統一する（R6 TD-M11 の第一段）。

## Scope

### 背景

`web/index.html` は `<html lang="ja">` 固定だが、
Reader の status には英語文言が残り、Editor / Settings の日本語と混在していた（TD-M11）。
`aria-live` 分離は ADR-0021 で明示的に却下済みで、
DOM 契約変更を伴うため第二段の別 ADR に委ねる。

### 成果物

- `web/src/lib.rs`: 変更（Reader status の英語文言を日本語へ）
- `tools/verify-web-reader.py`: 変更（E2E 期待値を日本語へ同期、総数 254 維持）
- `docs/decisions/0226-status-message-language.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. Reader の英語 status 文言を日本語へ置換する（動的値の書式は維持）。
2. 固定エラー文言はredaction 契約（ADR-0212）を維持し、詳細値を出さない。
3. E2E の文言依存（約 25 箇所）を同期更新し、`Checker(expected=254)` を維持する。
4. `aria-live` 属性と要素構造は変更しない（`role="status" aria-live="polite"` を維持）。
5. 全ゲート成功。

## Acceptance criteria

- E2E 254 項目、全ゲート成功。

## Constraints

- 新規依存なし（i18n クレートは追加しない）。
- `aria-live` 分離・error/info 分類表は ADR の未決定として残す。
- `master` へ push する。force push はしない。
