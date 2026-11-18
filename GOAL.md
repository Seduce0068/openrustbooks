# Goal

README の ADR 一覧で 0098〜0131 の説明が ADR-0132 に入れ子になっている問題を修正する（R6 PR-H07）。

## Scope

### 成果物

- `README.md`: ADR-0098〜0131 の説明 34 件を正しい行へ移動
- `todo.md`: PR-H07 を完了へ移動
- `GOAL.md`: 最終契約へ更新

### 決定

- `docs/decisions/index.md` を出典に、各 ADR 行へ 1 行 description を配置。
- ADR-0132 には自身の説明のみを残す。
- 0133 以降の追加は範囲外（README に未掲載の既存状態を維持）。

## Acceptance criteria

- markdownlint 0 issues（検証済み）。コード変更なし、E2E 248 維持。
