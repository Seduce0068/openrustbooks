# Goal

README / architecture の依存一覧を実装事実へ合わせる（ADR-0163 の未決定）。

## Scope

### 背景

ADR-0163 で pulldown-cmark の直接依存を削除したが、
README.md:74 と docs/architecture.md:178 は未使用の依存として記載したまま。
README.md:271 の ADR-0086 タイトルは履歴なので変更しない。

### 成果物

- `README.md`: 変更（依存一覧を `comrak` 中心に更新）
- `docs/architecture.md`: 変更（Markdown 処理の記述を更新）
- `docs/decisions/0163-remove-unused-markdown-dep.md`: 変更（未決定を解消）
- `GOAL.md`: 本契約

### 必須事項

1. README.md:74 の技術スタック表を「`comrak`（Markdown→HTML）, `ammonia`（サニタイズ）, `rumdl`（lint）」等、実装に合わせる。
2. docs/architecture.md:178 の `pulldown-cmark / comrak` も `comrak` 中心に更新（サニタイズと lint への言及を追加可）。
3. ADR-0163 の未決定「README / architecture の依存一覧更新」を解消済みにする。
4. markdownlint が通る。コード変更は行わない。

## Acceptance criteria

- markdownlint（全 docs + README）が 0 issues。

## Constraints

- ADR-0086 のタイトル（脆弱性修正の履歴）は変更しない。
- `todo.md` と `docs/investigation/` は触らない。
- `master` へ push する。force push はしない。
