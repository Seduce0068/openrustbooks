---
type: Decision
title: "ADR-0276: 一時調査成果物のスナップショットとしての扱い"
description: "未追跡の調査レポート群を当時スナップショットとして導線に明示し、生ファイルは変更しない決定を記録する。"
tags:
  - ADR
  - documentation
  - investigation
generated:
  by: dsh
  at: "2026-11-20T07:01:23+09:00"
status: accepted
---

## 背景

`docs/investigation/` には、別セッションが 2026-11-17 に実施した残タスク調査の
レポート群（round-01〜05 と final-report）が置かれている。成果は `todo.md` と ADR
（ならびに `docs/index.md` の導線）へ吸収済みだが、索引側の説明は
「別セッションによる残タスク調査レポート」とだけ記載され、未追跡の
一時成果物であることと、記載値が当時点のスナップショットであることが伝わらない。

本 ADR は、生ファイルを触らずに導線だけを実態へ合わせる決定を記録する。

## 現状（スナップショット）

final-report のメタデータと本文、各 round のフロントマターから次を確認した。

- 構成: `round-01.md` 〜 `round-05.md` と `final-report.md` の 6 ファイル。
- 生成者: すべて `generated.by: dsh`。
- 生成日: すべて 2026-11-17（`generated.at` は 20:14:45〜20:55:34 +09:00）。
- 最終確認時点: `HEAD = 38e6dac738d3122745fc7d087d9a6e30ebb2ee9e`。
- 当時の計数: ADR 83 件（0001〜0083、欠番なし）、
  E2E 219 件（`checker.check(` 202 + `checker.equal(` 17）。
- 当時の検証: 6 ゲート成功、`cargo test` 125 passed / 0 failed。
- 追跡状態: 作業ツリーは `?? docs/investigation/`（未追跡）。

これらは 2026-11-17 時点の値であり現行値ではない。現行は ADR-0276 追加後が
276 件、E2E が 256 件（ADR-0272）である。

## 決定

- `docs/investigation/` 配下の生ファイルは変更・コミットしない。
  [ADR-0083](0083-dead-code-cleanup.md) の「別セッションの調査レポートは
  変更・コミットしない」という決定を維持し、supersede しない。
- 変更するのは `docs/index.md` の導線のみとする。説明は
  「別セッションの一時調査スナップショット（未追跡・ADR-0083 で変更/コミットしないと
  決定）。数値は 2026-11-17 時点で現行ではない。成果は todo / ADR へ吸収済み」とする。
- レポート群は現行契約の根拠に使わず、追跡先は `todo.md`・ADR・
  `docs/index.md` に一本化する。
- E2E 256 件・check 名・コード・`GOAL.md` は本決定で変更しない。

## 根拠

- [ADR-0083](0083-dead-code-cleanup.md): 別セッション成果物を変更・コミットしない
  明文の決定。round-01 を背景根拠としつつ、レポート自体は触らない。
- [ADR-0171](0171-clear-data-confirmation.md): final-report の C03 を背景として
  参照し、結論を ADR として吸収した先例。レポートは根拠の手がかりであり、
  現行契約そのものではない。
- [ADR-0267](0267-bundle-size-baseline.md): `docs/investigation/round-02.md:65` の
  22.99 MB / gzip 2.53 MB を「条件不明の参考値・非比較」として扱い、
  verify.sh と同一条件で再計測した先例。スナップショット数値を現行値として
  流用しない判断の根拠。
- [final-report](../investigation/final-report.md) の自己申告: 本レポートは
  調査ループのメタ成果物であり `GOAL.md` の実行契約には依存しない、レポート群は
  未追跡である、追跡対象への移行はオーナーが判断する、と明記されている。
- `todo.md` の吸収済み記録: 5 ラウンドと第 6 回統合の結果、却下済み誤検知、
  未確認項目、R6 での ADR 実績照合が完了として記録済み。

## 未決定

- 追跡昇格が必要になった場合の手順。どのファイルを追跡対象にするか、
  stale 注記の文言と付与先、昇格の判断基準は未決定。昇格には
  ADR-0083 の supersede が必要であり、本 ADR は行わない。
- 公開リポジトリでの開発ホストパス除去。
  `docs/investigation/round-05.md:27` には `strings | grep /home/koppe` の
  実測例が残る。ADR-0083 を維持する限り除去できないため、公開時に
  除去する場合は supersede の判断が必要。なお配布 WASM 自体は
  ADR-0267 の実測で `/home/koppe` の出現が 0 件であることを確認済みで、
  これは調査レポート内の文字列とは別の事柄である。

## 読み方の注意

- 却下済みの誤検知を現行事実として読まないこと。例:
  「capability 不在で IPC 全拒否」は Tauri 2.12.5 が独自コマンドを ACL 非対象と
  する点から却下され、真因は A05（invoke 不在）とされた。現在は ADR-0270 で
  グローバル IPC 経路が確定し、ADR-0272 で desktop の UI 接続まで完了している。
  他にも「core にサイズ上限なし」「SVG 内 CSS が対象外」「MD013 違反 79 行」
  「Pillow 依存が未記載」「Safari が forced-colors 非対応」が却下済みである。
- 未確認として載っている項目（GitHub issue / PR の状態、CI の実際の実行状態、
  EPUB 2 / 固定レイアウト、外部脆弱性情報の照合、Tauri v2 capability の実挙動）
  は当時の未確認であり、現行の未確認事項ではない。
- `HEAD = 38e6dac`、ADR 83 件、E2E 219 件はすべて 2026-11-17 時点の
  スナップショットであり、現 HEAD・現行件数として引用しない。

## 影響

- `docs/investigation/` 配下の 6 ファイルは未変更のまま。
- コード、E2E 256 件・check 名、`GOAL.md` は未変更。
- `docs/index.md`、`docs/decisions/index.md`、`docs/log.md`、`README.md`、
  `todo.md` を追従（ADR は 276 件）。

## 検証

- `npx markdownlint-cli2 'docs/**/*.md' README.md todo.md`: 0 issues。
- 本 ADR と各索引のリンク先ファイルが存在することを確認。
- `docs/investigation/` は読み取りのみで、書き込みを行っていない。

## 参照

- [ADR-0083: デッドコードと古い契約値の整理](0083-dead-code-cleanup.md)
- [ADR-0171: Clear data の確認ダイアログ](0171-clear-data-confirmation.md)
- [ADR-0267: WASM バンドルサイズの比較可能ベースライン](0267-bundle-size-baseline.md)
- [ADR-0272: Tauri 設定保存 UI の接続](0272-settings-save-ui.md)
- [final-report（一時調査スナップショット）](../investigation/final-report.md)
- [docs/index.md](../index.md) / [decisions/index.md](index.md) / [todo.md](../../todo.md)
- [ドキュメント運用規約](../documentation-operations.md)
