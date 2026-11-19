---
type: Decision
title: "ADR-0241: generated.at の意味論と日付逆転の扱い"
description: "ADR の generated.at は生成・正規化時点の記録であり作成時刻ではないと明記し、既存メタデータを保持する決定。"
tags:
  - ADR
  - documentation
  - okf
  - timestamp
generated:
  by: dsh
  at: "2026-11-19T20:21:50+09:00"
status: accepted
---

## 現状

- ADR-0001 / 0002 の `generated.at` は `2026-11-17T13:19:26+09:00`。
  frontmatter 正規化（c030537）の際に `generated` が付与された値である。
- ADR-0003 は `2026-10-16T01:12:27+09:00`、0004 は `2026-10-17T15:20:00+09:00`、
  0005 は `2026-10-17T16:50:00+09:00`。
- git 上の作成順は 0001: 2026-10-11 → 0002: 2026-10-12 → 0003: 2026-10-16 →
  0004 / 0005: 2026-10-17 で、連番と一致する（[GOAL.md](../../GOAL.md) の契約事実）。
- したがって日付の見かけ上の逆転は `generated.at` を「作成時刻」と読んだことによる誤解であり、
  決定順の矛盾ではない。`GOAL.md` は値の訂正が不要で規約の注記が必要であることを契約する。

## 決定

- 既存 240 件の ADR メタデータ（`generated.at` を含む）は変更しない。
- `docs/documentation-operations.md` の RFC 3339 節へ `generated.at` の意味論を明記する。
  - `generated.at` は生成／正規化した時刻で、作成時刻や決定順ではない。
  - ADR の連番は決定の依存順、作成時刻の根拠は git 履歴。
  - 既存メタデータは事実として保持し、見かけの日付逆転は訂正対象ではない。
  - `created` キーの導入／一括移行は未決定。
- 本 ADR は規約の注記であり、メタデータのスキーマ変更を伴わない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 値を訂正（作成時刻へ書き換え） | 却下。正規化時に付与された事実を破壊し、既存 240 件不変の契約に反する |
| git proxy 書き込み（git log の日付を generated.at に反映） | 却下。`generated` の意味と矛盾し、一括移行を伴う |
| 規約注記（採用） | 値を変えず誤読を解消。事実保持と誤解防止を両立し、変更範囲が最小 |

`generated.at` の値はその文書が生成・正規化された時点を記録する。
正規化は後から行われるため、連番の古い ADR により新しい時刻が入るのは自然な挙動である。
作成時刻を知りたい場合は git 履歴を参照する。

## 検証

- git 履歴の作成年月日（再現手順）:

  ```bash
  git log --diff-filter=A --date=short --format='%ad %h %s' -- \
    docs/decisions/0001-*.md docs/decisions/0002-*.md docs/decisions/0003-*.md \
    docs/decisions/0004-*.md docs/decisions/0005-*.md
  ```

  期待値は 0001: 2026-10-11、0002: 2026-10-12、0003: 2026-10-16、
  0004 / 0005: 2026-10-17（本件では `GOAL.md` の契約事実を根拠とし、git 操作は行わない）。
- `docs/decisions/index.md` は ADR-0001 から ADR-0241 まで連番順に列挙し、
  欠番・重複なしの全 241 件とする。
- `npx markdownlint-cli2 'docs/**/*.md' README.md todo.md` は 0 issues。
- 既存 ADR 240 件の `generated.at` は不変（本件では 0001〜0005 の値を読み取って確認）。

## 未決定

- `created` キーの導入（作成時刻を明示する専用キーの定義）。
- 既存 240 件の ADR への `created` 一括移行／バックフィル。
- 注記の index 配置（各 ADR 項目へ注記を添えるか、規約側へ集約するか。
  現状は規約側へ集約し、index は連番と要約のみとする）。
- git commit を `generated` の proxy（作成時刻の代用）として扱うか。

## 参照

- [docs/documentation-operations.md](../documentation-operations.md)
- [ADR-0234: ドキュメント運用の一本化](0234-documentation-operations.md)
- [ADR-0001: デスクトップ版にOS WebViewを採用する](0001-use-system-webview.md)
- [ADR-0002: AI支援Markdown生成・修正パイプラインを導入する](0002-ai-markdown-pipeline.md)
- [ADR-0003: 共通コアをターゲット別依存でNative/Wasm両対応にする](0003-native-wasm-target-split.md)
- [ADR-0004: ZIPアーカイブ閲覧をcoreの読み取り専用モジュールとして実装する](0004-zip-archive-reader.md)
- [ADR-0005: EPUB本文の読み出しをcoreの読み取り専用モジュールとして実装する](0005-epub-body-reading.md)
- [GOAL.md](../../GOAL.md)
