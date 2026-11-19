---
type: Reference
title: "ドキュメント運用規約"
description: "OKF・索引・log・ADR・lint の運用をリポジトリ内へ一本化する手順。"
tags:
  - documentation
  - okf
  - markdownlint
  - adr
generated:
  by: dsh
  at: "2026-11-19T18:10:29+09:00"
status: accepted
---

## 適用範囲

本規約はリポジトリ内で成果物として残す Markdown に適用する。

- `docs/` 配下のすべてのファイル（`docs/decisions/` の ADR を含む）
- `README.md`
- `todo.md`

チャット応答や一時ファイルは対象外。正本は `~/.agents/AGENTS.md` セクション 8
（成果物の文書形式）で、本ファイルは openrustbooks 内の具体的な手順へ落とし込んだもの
（[ADR-0234](decisions/0234-documentation-operations.md)）。

## OKF フロントマター

成果物ファイルは先頭に YAML フロントマターを置く。キーは次のとおり。

| 区分 | キー | 内容 |
| --- | --- | --- |
| 必須 | `type` | `Reference` / `Decision` / `Playbook` / `Runbook` / `Todo` など |
| 推奨 | `title` / `description` / `tags` | タイトル、1 文の要約、タグ（ADR は `ADR`、他は小文字 kebab-case） |
| 任意 | `resource` / `generated` / `verified` / `status` / `stale_after` | 出典 URL、生成者と時刻、検証者と時刻、状態、陳腐化時刻 |

- `generated` / `verified` は `by`（識別子）と `at`（時刻）の入れ子で記録する。
- `status` は ADR では `accepted` を用いる。他の文書は必要に応じて設定する。
- 文書を探すときは対象ディレクトリの `index.md` を先に読み、
  `type` / `tags` / `status` で候補を絞る。索引は候補選びの材料であり内容の保証ではない。

## RFC 3339 と日付の確認

日付・時刻は RFC 3339 に準拠し、UTC オフセットを必ず含める。

- 正: `2026-11-19T18:10:29+09:00` / `2026-11-19T09:10:29+00:00`
- 誤: `Z` 接尾辞、`2026-11-19` のような日付のみ、タイムゾーン名（`JST` など）

「現在の日付」は自己認識で断定せず、`date` コマンドで確認する。

```bash
date --iso-8601=seconds
date -u --iso-8601=seconds
```

### generated.at の意味論

`generated.at` は文書を生成／正規化した時刻であり、文書の作成時刻や決定順ではない。
ADR-0001 / 0002 の `generated.at` が 2026-11-17、0003〜0005 が 2026-10-16〜17 なのは、
0001 / 0002 に frontmatter 正規化時（c030537）の `generated` が付与されたためである。
git 上の作成順は 0001: 2026-10-11、0002: 2026-10-12、0003: 2026-10-16、
0004 / 0005: 2026-10-17 で、連番と一致している。

- ADR の連番は決定の依存順を表す。作成時刻の根拠は `git log` の履歴であり、
  `generated.at` から推測しない。
- 既存メタデータは付与時の事実として保持する。日付の見かけ上の逆転は意味論の誤読であり、
  訂正対象ではない（[ADR-0241](decisions/0241-generated-timestamp-semantics.md)）。
- `created` キーの導入と既存 ADR への一括移行は未決定とし、本規約では定義しない。

## 見出し階層

- `title` を持つ文書では markdownlint が `title` を H1 として扱う。
  本文の最初の見出しは `##` から始め、`##` → `###` → `####` の階段を守り、
  本文に `#` は使わない（MD025 / MD041）。
- `index.md` / `log.md` は例外。フロントマターを持たず、単一の `#` 見出しから始める。
- 見出しの前後には空行を置き、見出しレベルを飛ばさない（MD022 / MD001）。
- 基本は CommonMark。表・タスクリスト・コードブロックの言語指定は GFM に従う。

## index.md と log.md

- `index.md` は概念文書を収容する各ディレクトリに置き、
  配下へのリンクと `description` を列挙する。
- `log.md` は変更を加えたコミットで追記する。日付見出しは `## YYYY-MM-DD`
  形式で新しい順に並べ、各項目の末尾に根拠 ADR を添える。
- 自動生成する索引は HTML コメント（`<!-- ... -->`）で領域を区切り、
  区切られた範囲内のみを更新する。手動項目と混在させない。

## ADR の追加手順

ADR は `docs/decisions/NNNN-*.md`（4 桁連番）として追加し、
OKF `type: Decision` と `##` 見出しで記述する。追加時は次をすべて更新する。

1. `docs/decisions/index.md` に見出し・リンク・`status`・要約を追記し、件数を更新する。
2. `docs/index.md` に ADR へのリンクと要約を追記する。
3. `docs/log.md` の先頭日付見出しへ変更内容を追記する。
4. `README.md` の「ドキュメント」セクションへリンクと要約を追記する。

## 要件・E2E トレーサビリティ

- VC / REQ と E2E check 名の対応は `docs/e2e-traceability.md` を正とする。
- check 名は `tools/verify-web-reader.py` を Python AST で解析し、
  `checker.check` / `checker.equal` の第 1 引数（文字列リテラル）を完全一致で記載する。
- E2E 総数はコードの `Checker(expected=...)` を出典とし、文書側で手計算しない。
  コード出典への固定は [ADR-0090](decisions/0090-e2e-expected-count.md)、
  同名 check の重複検知は [ADR-0182](decisions/0182-e2e-unique-check-names.md)、
  253 件への追従は [ADR-0220](decisions/0220-traceability-refresh.md)、
  254 件（check 237 + equal 17）への更新は
  [ADR-0225](decisions/0225-broken-image-settling.md) が根拠。

## markdownlint

リポジトリ直下の `.markdownlint-cli2.jsonc` が設定の正本で、
`/tmp/mdlint.jsonc` のような一時ファイルには依存しない。CLI は設定ファイルを
自動検出するため、次のコマンドで検証する。

```bash
npx markdownlint-cli2 'docs/**/*.md' README.md todo.md
```

明示的に指定する場合は次を使う。結果は 0 issues を必須とする。

```bash
npx -y markdownlint-cli2 --config .markdownlint-cli2.jsonc \
  'docs/**/*.md' README.md todo.md
```

設定は `default: true`。MD013 は行長 120（コードブロック・表・見出しは除外）、
MD041 はフロントマターの `title` / `name` / `description` を H1 として扱い、
MD060 は一貫したスタイルを要求する。

## 参照

- [ADR-0234: ドキュメント運用の一本化](decisions/0234-documentation-operations.md)
- [E2E トレーサビリティ](e2e-traceability.md)
- [docs/index.md](index.md) / [docs/log.md](log.md) / [decisions/index.md](decisions/index.md)
