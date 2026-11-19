# Goal

ドキュメント運用（OKF / 索引 / log / lint）をリポジトリ内へ明文化する（R6 C04 / R5-ops#1）。

## Scope

### 背景

`docs/index.md` / `docs/decisions/index.md` / `docs/log.md` は整備済みだが、
OKF フロントマター、RFC 3339、見出し階層、索引更新、markdownlint 設定の
運用がリポジトリ内に一本化されていない。
lint 設定は `/tmp/mdlint.jsonc` に依存しており再現性が弱い。

### 成果物

- `.markdownlint-cli2.jsonc`: 追加（リポジトリ内 lint 設定）
- `docs/documentation-operations.md`: 追加（OKF / 索引 / log / ADR / lint 手順）
- `AGENTS.md`: 変更（グローバル規約への参照を現行パスへ更新）
- `docs/decisions/0234-documentation-operations.md`: 追加
- `docs/index.md` / `README.md` / `docs/decisions/index.md` / `docs/log.md` / `todo.md`: 更新
- `GOAL.md`: 本契約

### 必須事項

1. OKF 必須キー・RFC3339 オフセット・`##` 見出し・index/log 予約ファイルの規約を記載する。
2. ADR 追加時の `docs/decisions/index.md` / `docs/index.md` / `docs/log.md` 更新手順を明記する。
3. `.markdownlint-cli2.jsonc` を追加し、`npx markdownlint-cli2 'docs/**/*.md' README.md todo.md` が 0 issues になることを確認する。
4. `AGENTS.md` のグローバル参照を `~/.agents/AGENTS.md`（現行正本）へ更新する。
5. C04 を完了へ移動し、既存 ADR / 索引 / log を更新する。

## Acceptance criteria

- markdownlint 0 issues、verify.sh（E2E 254）成功。

## Constraints

- グローバル AGENTS.md 自体は変更しない。
- 既存 ADR / 要件 / 実装は変更しない。
- `master` へ push する。force push はしない。
