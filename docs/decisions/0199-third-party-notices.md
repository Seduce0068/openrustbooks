---
type: Decision
title: "ADR-0199: 第三者通知の網羅"
description: "配布に関係する依存クレートを cargo metadata から網羅し、ライセンス ID と出典を第三者通知へ記載する決定を記録する。"
tags:
  - ADR
  - docs
  - deps
generated:
  by: dsh
  at: "2026-11-19T08:47:00+09:00"
status: accepted
---

## 背景

`THIRD_PARTY_NOTICES.md` は旧 MIT 本文と imagesize 0.15.0 のみを記載しており、
配布 WASM / デスクトップに含まれる依存クレートの通知が無かった（R6 PR-M12）。
AGPL-3.0-or-later 変更後も上流の著作権表示・許諾通知を保持する方針が
ファイルに書かれていた。

## 決定

- 出典は `cargo metadata --locked --format-version 1` と `Cargo.lock` とする。
- workspace ルート 3 クレート（core / desktop / web）から、
  `dep_kinds[].kind` が `null`（通常）または `build` のエッジのみを辿り、
  外部依存 781 件を列挙する。
- dev-only の 9 件（pretty_assertions / tokio-test / wasm-bindgen-test 等）は除外する。
- 表の列は「クレート名 / バージョン / ライセンス ID / 出典 URL」とする。
  - ライセンス ID が無い場合は「未確認」とし、推測で補完しない（実際は 0 件）。
  - crates.io 由来のみを掲載する。
- 旧 MIT 本文と imagesize 通知は履歴として保持する。
- 自動生成セクションを
  `<!-- BEGIN AUTO-GENERATED DEPENDENCIES -->` /
  `<!-- END AUTO-GENERATED DEPENDENCIES -->` で囲む。
- ライセンス本文は追加せず、ID と出典のみを記載する
  （full text は各クレートの配布物を参照）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| cargo metadata から生成（採用） | 依存グラフを機械的に網羅できる |
| 手作業で主要依存のみ記載 | 漏れが生じる |
| cargo-deny を導入 | 今回は新規ツール追加を避ける（後述） |

## 影響

- `THIRD_PARTY_NOTICES.md` は 864 行／781 行の依存表を含む。
- 121 件は target 限定エッジ（windows / macos / wasm 等）経由であり、
  本一覧は「all-target のスーパーセット」（過剰側に安全）。
- 35 件は `MIT/Apache-2.0` 等の非 SPDX スラッシュ表記を
  metadata どおり転記している（正規化しない）。
- 旧 imagesize 行の URL のみ MD034 対応で autolink 化した
  （リンク先文字列は不変）。
- 新規依存・コード変更なし、E2E 251 維持。

## 検証（Runtime Proof）

- 独立再計算で 781 件一致（missing 0 / extra 0）。
- dev-only 9 件はすべて除外、path 依存・first-party の混入なし。
- markdownlint（設定ファイル指定／デフォルト）0 issues。
- `./scripts/verify.sh` は既存の緑を維持（E2E **251 項目すべて成功**）。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- full license text を通知へ同梱する法的要否（現状は ID + URL のみ）。
- target 別の依存抽出（現状は kind のみ・all-target スーパーセット）。
- 非 SPDX スラッシュ表記の正規化ルール。
- 生成スクリプトの `tools/` 常設とマーカー外のバイト単位保持。
- cargo-deny の導入（環境にバイナリはあるが `deny.toml` は無く、
  本次では未使用。GOAL の「既存ツール」表記は ADR で訂正）。
