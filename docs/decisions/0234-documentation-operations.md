---
type: Decision
title: "ADR-0234: ドキュメント運用の一本化"
description: "OKF・索引・log・lint の規約と設定をリポジトリ内へ集約する決定。"
tags:
  - ADR
  - documentation
  - okf
  - markdownlint
generated:
  by: dsh
  at: "2026-11-19T18:10:29+09:00"
status: accepted
---

## 目的

OKF フロントマター、RFC 3339、見出し階層、索引・log の更新手順、markdownlint 設定を
リポジトリ内に一本化し、セッションやハーネスに依存せず再現できる文書運用を確立する。
`GOAL.md`（R6 C04 / R5-ops#1）の契約を満たす。

## 背景

- `docs/index.md` / `docs/decisions/index.md` / `docs/log.md` は整備済みだが、
  運用規約はグローバル `~/.agents/AGENTS.md` セクション 8 に分散し、
  リポジトリ内には無かった。
- lint 設定は `/tmp/mdlint.jsonc` に依存しており、一時ファイルの消失で再現性が失われる。
- `AGENTS.md` は旧パス `~/.config/opencode/AGENTS.md` を参照しており、
  現行の正本 `~/.agents/AGENTS.md` と食い違っていた。
- ADR 追加時の索引・log・README の更新手順が暗黙知になっていた。

## 決定

### 規約と設定の配置

- `docs/documentation-operations.md` を運用規約のリポジトリ内正本とする。
  適用範囲は `docs/`（ADR を含む）、`README.md`、`todo.md`。
- `.markdownlint-cli2.jsonc` を追加し、`/tmp/mdlint.jsonc` と同等の設定を固定する。
  `default: true`、MD013 は行長 120（コードブロック・表・見出しは除外）、
  MD041 はフロントマターの `title` / `name` / `description` を H1 として扱い、
  MD060 は一貫したスタイルを要求する。
- `AGENTS.md` のグローバル参照を `~/.agents/AGENTS.md`（正本）へ更新する。
  プロジェクト外のグローバル TODO へのパス言及は削除し、
  プロジェクト固有の TODO は本リポジトリの `todo.md` を正とする。

### 文書形式

- OKF の必須キーは `type` のみ。推奨は `title` / `description` / `tags`、
  任意は `resource` / `generated` / `verified` / `status` / `stale_after`。
- 日付・時刻は RFC 3339 に準拠し UTC オフセットを含む。現在日時は `date` で確認する。
- `title` を持つ文書は本文の最初の見出しを `##` とし、本文に `#` を使わない。
  `index.md` / `log.md` はフロントマターなしの単一 `#` とする例外。
- `index.md` は配下リンクと `description` を列挙する。
  `log.md` は変更コミットで `## YYYY-MM-DD` を新しい順に追記する。

### ADR 追加時の更新

`docs/decisions/NNNN-*.md` を追加した際は、
`docs/decisions/index.md`（件数と項目、自動生成領域は HTML コメントで区切る）、
`docs/index.md`、`docs/log.md`、`README.md` を同じコミットで更新する。
VC / REQ と E2E の対応は `docs/e2e-traceability.md` を正とし、
E2E 総数は `Checker(expected=...)` を出典とする。

### 検証

`npx markdownlint-cli2 'docs/**/*.md' README.md todo.md`（設定ファイル自動検出）を
実行し、0 issues を完了条件とする。

## 根拠

- 共通指示セクション 8 が OKF / RFC 3339 / 見出し階層 / 予約ファイル / lint を規定しており、
  本 ADR は其内容をプロジェクト内へ写したもので、新たな形式を導入しない。
- `Checker(expected=254)` は `tools/verify-web-reader.py:845` の実測であり、
  E2E 総数をコード出典にする既存契約（ADR-0090 / 0182 / 0220 / 0225）を維持する。
- 設定ファイルをリポジトリ直下へ置くことで、一時ファイルの有無に左右されない
  同一の検証を誰でも再現できる。CLI 側の設定自動検出により日常コマンドは短いままである。
- 既存 ADR・要件・実装・E2E は変更しない。本件は文書運用と lint 設定のみの変更である。

## 影響

- 新規ファイル: `.markdownlint-cli2.jsonc`、`docs/documentation-operations.md`、本 ADR。
- 更新: `AGENTS.md`、`docs/index.md`、`docs/decisions/index.md`、`docs/log.md`、
  `README.md`、`todo.md`。ADR 件数は 233 → 234 件。
- C04 / R5-ops#1 を「未着手」から「完了」へ移動する。
- 署名・配布・コード・ゲート構成への変更はない。E2E 254 件は不変。

## 未決定

- lint 設定の CI ゲート化: `Seduce0068/workflows/rust-gates.yml@v1` の
  `extra_gates` へ markdownlint-cli2 を組み込むかは未決定。現状はローカル実行の規約のみ。
- 旧パス互換: `~/.config/opencode/AGENTS.md` や `/tmp/mdlint.jsonc` への
  シンボリックリンク等による互換維持は行わない。参照を現行パスへ統一した。
  他ハーネスからの旧パス参照が残る場合の移行手順は未決定。
- 自動生成索引の Range コメント運用: HTML コメントによる領域区切りは方針として
  定めたが、具体的な開始・終了マーカの書式、生成ツール、手動項目との併用規則は未決定。
  現状の `docs/decisions/index.md` は手動運用のままとする。

## 参照

- [docs/documentation-operations.md](../documentation-operations.md)
- [ADR-0090: E2E 期待総数のコード出典化](0090-e2e-expected-count.md)
- [ADR-0182: E2E check 名の一意性](0182-e2e-unique-check-names.md)
- [ADR-0220: E2E トレーサビリティの 253 件への更新](0220-traceability-refresh.md)
- [ADR-0225: 壊れた画像のセトリング待ち解消](0225-broken-image-settling.md)
- [ADR-0233: 既存ソリューション選定根拠の横断記録](0233-solution-selection-rationale.md)
- [docs/e2e-traceability.md](../e2e-traceability.md)
- [GOAL.md](../../GOAL.md)
