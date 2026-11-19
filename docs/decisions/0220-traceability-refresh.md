---
type: Decision
title: "ADR-0220: E2E トレーサビリティの 253 件への更新"
description: "ADR-0206 の追従漏れを解消し、253 件の E2E check 名を VC 対応表へ完全一致で反映する決定。"
tags:
  - ADR
  - requirements
  - testing
  - traceability
generated:
  by: dsh
  at: "2026-11-19T14:23:43+09:00"
status: accepted
---

## 背景

[ADR-0206](0206-e2e-traceability.md) は検証条件（VC）と E2E check 名の対応表を
251 件で作成した。その後 [ADR-0212](0212-status-error-redaction.md) が
「不正 EPUB は固定文言のみ通知しサンプル EPUB は通常どおり表示される」を、
[ADR-0213](0213-reduced-motion.md) が
「Reduce で色 transition を保ち空間移動なしでフラグメントへ即時移動する」を追加し、
期待総数は 252 → 253 へ進んだ。しかし `docs/e2e-traceability.md` の対応表と
概要・集計は 251 のままで、要件から E2E への追跡が途中までしかできなかった。

`GOAL.md` は本件を契約として固定し、AST による 253 件の抽出と
missing / duplicate / extra 0 を要求している。

## 決定

- 対応表へ次の 2 件を追加し、AST 抽出結果 253 件と完全一致させる。
  - 不正 EPUB の固定文言通知 → **VC-009** の既存行（エラー通知）へ追加。
  - reduce の即時フラグメント移動 → **VC-004** の既存行
    （章移動・リンク解決・フラグメント履歴）へ追加。
- 概要・集計・参照・`docs/index.md` の件数を 251 → 253 へ更新する。
- 期待総数はコード出典とし、`tools/verify-web-reader.py:843` の
  `Checker(expected=253)` を参照する（[ADR-0090](0090-e2e-expected-count.md) を維持）。
  内訳は AST 実測の checker.check 236 + checker.equal 17 = 253 とする。
- check 名の抽出は一時ファイル（`docs/e2e-traceability.md` 旧記載の
  `/tmp/e2e-check-names.txt`）に依存せず、Python AST を直接手順として記載する。
- E2E コード・check 名・期待総数は変更しない。
- 履歴 ADR と完了 todo の 251 件表記は、作成時点の実績として保持する。

## 根拠

- **ADR-0206 の追従漏れ**: 対応表が 251 で止まり、以降の 2 件が未反映だった。
  放置すると ADR-0090 の期待総数ゲートが通っても要件追跡が欠落する。
- **ADR-0212 / ADR-0213 の E2E 追加**: 双方とも E2E を 1 件ずつ追加し、
  期待総数を 252 / 253 へ更新した。対応表も同じ時点へ追従させる。
- **ADR-0090 のコード出典方針**: 期待総数はコードを権威とする。
  したがって文書側はコードから AST 抽出した実名へ合わせ、
  文書からコードへ数値を逆入力しない。
- **VC 割当て（不正 EPUB）**: 本 check の固有主張は、status / console へ
  内部エラーの Debug 表現を出さず固定文言のみを通知することである。
  ADR-0212 は redaction（情報漏洩防止）の決定で、対応表の VC-009 行は
  status / console のエラー通知を既に集約しているため、そこへ追加する。
  サンプル EPUB の再表示はリーダー側の回帰ガードであり、主眼ではない。
  VC-004（リーダー）も検討したが、対応サンプルのオープン成功経路を集合とする
  VC-004 より、通知文言の固定化を扱う VC-009 が適合すると判断した。
- **VC 割当て（reduce）**: 本 check はフラグメントへの即時移動と
  位置適用（scroll 再試行のスキップ）を検証するもので、
  VC-004 の章間移動・フラグメント履歴・読書位置の復元の領域にある。
  VC-012 は画像読み込み・CLS・サイズ警告の VC で主眼が異なる。
  VC-015 は実機 5 ブラウザ（Chrome / Edge / Brave / Firefox / Safari）の
  未カバー項目であるが、本 check は単一 CDP エンジン上で
  `Emulation.setEmulatedMedia` により reduce をエミュレートした自動 E2E であり、
  実機検証ではないため割り当てない。
- **内訳 236 + 17**: ADR-0212 時点の checker.check 235 + checker.equal 17 = 252 は
  その時点の実績であり、ADR-0213 の 1 件追加で現在は 236 + 17 = 253 となる。
  AST 実測値を採用する。

## 影響

- 更新対象は `docs/e2e-traceability.md`、`docs/index.md`、`README.md`、
  `docs/decisions/index.md`、`docs/log.md`、`todo.md` と本 ADR に限定する。
- E2E の挙動・check 名・期待総数は不変で、コード変更を伴わない。
- 旧 251 件の記述は ADR-0206 と完了 todo に履歴として残る。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- Python AST 抽出: `ast.parse` で `checker.check` / `checker.equal` の第 1 引数を収集し、
  check 236 + equal 17 = **253** 件、一意な check 名 253、重複 0 を確認。
- 対応表との照合: 更新前は missing 2 / duplicate 0 / extra 0、
  更新後は **missing 0 / duplicate 0 / extra 0**（253 / 253 一致）。
- markdownlint-cli2（`--config /tmp/mdlint.jsonc`）: 全対象 **0 issues**。
- E2E 253 / 0 の実測は ADR-0213 および ADR-0219 の記録を参照する。
  本エージェントの書き込み範囲は文書のみのため verify.sh は再実行していない。

## 未決定

- 要件 VC の再分割（VC-004 が広範なため、エラー通知と reduce を独立 VC に
  分割するか、現状の集約を維持するか）。
- 実機 5 ブラウザ（Chrome / Edge / Brave / Firefox / Safari）での検証。
- 今後の E2E 追加時に、AST 抽出と対応表の一致を自動追従させる仕組み
  （CI ゲート化または生成スクリプトの導入）。

## 参照

- [GOAL.md](../../GOAL.md) — 本件の契約
- [ADR-0206: E2E トレーサビリティの可視化](0206-e2e-traceability.md)
- [ADR-0090: E2E 期待 check 総数の検証](0090-e2e-expected-count.md)
- [ADR-0212: Reader エラー status の redaction](0212-status-error-redaction.md)
- [ADR-0213: prefers-reduced-motion の尊重](0213-reduced-motion.md)
- [E2E トレーサビリティ](../e2e-traceability.md)
- [todo.md](../../todo.md) — 完了記録
