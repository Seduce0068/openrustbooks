# Goal

docs 配下に `index.md` と `log.md` を整備する（todo.md C04 / R5-ops#1）。

## Scope

### 背景

共通指示では概念文書ディレクトリに `index.md`、変更を加えたコミットで
`log.md` への追記を求めているが、`docs/` と `docs/decisions/` の両方に
存在しない。`docs/decisions/` には 90 件の ADR があり、索引が必要。

### 成果物

- `docs/index.md`: 追加（docs 配下の文書一覧）
- `docs/decisions/index.md`: 追加（全 ADR のタイトル・status・.description 一覧）
- `docs/log.md`: 追加（初回エントリ）
- `GOAL.md`: 本契約

### 必須事項

1. `docs/index.md` に `requirements.md` / `architecture.md` / `decisions/` /
   `investigation/` へのリンクと説明を列挙する。
2. `docs/decisions/index.md` に全 90 件の ADR を
   `- [ADR-XXXX](file.md) — status: ... — description` の形式で列挙する。
   タイトルは frontmatter の `title`、status は `status`、説明は `description` を使う。
3. `docs/log.md` に `## 2026-11-17` の見出しと、今回の索引整備の記録を追加する。
4. 予約文件格式に従い、フロントマターを付けず単一の `#` 見出しから始める。
5. markdownlint が通る。

## Acceptance criteria

- markdownlint が全ファイルで 0 issues。
- 6 ゲート・E2E が成功する。

## Constraints

- ADR 本体は変更しない。
- `docs/investigation/` は別セッションの成果物のため内容は変更しない
  （`docs/index.md` からのリンクのみ追加）。
- `master` へ push する。force push はしない。
