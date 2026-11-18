# Goal

README の ADR 一覧で 0098〜0131 の説明が ADR-0132 に入れ子になっている問題を修正する（R6 PR-H07）。

## Scope

### 背景

README.md の ADR 一覧は 0098〜0131 がリンクのみで、本来それぞれの説明が
ADR-0132 の行の下に 35 件ぶん入れ子になっている。
`docs/decisions/index.md` には正しい description があるため、これを出典にして
各 ADR 行へ 1 行の説明を戻す。

### 成果物

- `README.md`: 変更（ADR 0098〜0131 の説明を正しい行へ配置）
- `docs/decisions/0132-preserve-logic-unification.md`: 必要なら未決定の解消記録
- `docs/decisions/0179-editor-autosave.md`: 変更不要（索引は最新）
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. ADR-0098〜0131 の各行に `docs/decisions/index.md` の description を 1 行で追加する。
2. ADR-0132 の行には ADR-0132 自身の description のみを残す。
3. 既存のリンク・見出し・ライセンス節を壊さない。
4. markdownlint が通る。
5. コード変更なし（E2E 248 維持）。

## Acceptance criteria

- markdownlint（docs + README + todo）が 0 issues。

## Constraints

- `docs/decisions/index.md` は自動生成の出典として変更しない。
- `todo.md` は R6 の成果物として保持し、完了反映のみ行う。
- `master` へ push する。force push はしない。
