# Goal

doc-scanner の指摘に基づき、README とドキュメント索引の陳腐化を解消する。

## Scope

### 背景

- README の構成ツリーが `LICENSE # MIT License` としているが、本文末尾と
  LICENSE 実体は `AGPL-3.0-or-later`（リモートの b44b063 で変更済み）。
- README の ADR 一覧は 0084 止まり、docs/index.md と decisions/index.md は
  「全 90 件」とし、decisions/index.md は 0090 まで（実際は 0094 まで 94 件）。

### 成果物

- `README.md`: 変更（LICENSE コメント、ADR 一覧を 0094 まで）
- `docs/index.md`: 変更（件数を 94 に）
- `docs/decisions/index.md`: 変更（0091〜0094 を追加、件数を 94 に）
- `GOAL.md`: 本契約

### 必須事項

1. README の `LICENSE # MIT License` を `AGPL-3.0-or-later` に変更。
2. README の ADR 一覧に 0085〜0094 を追加。
3. 両 index の件数を 94 に更新し、decisions/index.md に 0091〜0094 を追加。
4. markdownlint が通る。

## Acceptance criteria

- ドキュメントと実体が一致する。
- 6 ゲート・E2E が成功する。

## Constraints

- ADR 本体は変更しない。
- `docs/investigation/` と `todo.md` は別セッション成果物のため触らない。
- `master` へ push する。force push はしない。
