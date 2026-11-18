# Goal

`docs/index.md` の ADR 件数表記を実態へ合わせる。

## Scope

### 背景

`docs/index.md:7` は「全 90 件の索引」とあるが、
`docs/decisions/` には ADR-0169 まで（169 件）存在する。
監査で繰り返し指摘されている陳腐化（ADR-0159 監査など）。

### 成果物

- `docs/index.md`: 変更（169 件へ更新）
- `GOAL.md`: 本契約

### 必須事項

1. ADR 件数を実際のファイル数（169）へ更新する。
2. markdownlint が通る。
3. 他のドキュメント表現は変更しない。

## Acceptance criteria

- markdownlint（全 docs + README）が 0 issues。

## Constraints

- `todo.md` と `docs/investigation/` は触らない。
- `master` へ push する。force push はしない。
