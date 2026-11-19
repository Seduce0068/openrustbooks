---
type: Decision
title: "ADR-0206: E2E トレーサビリティの可視化"
description: "要件の検証条件に VC ID を付与し、E2E check 名との対応表を追加する決定を記録する。"
tags:
  - ADR
  - testing
  - docs
generated:
  by: dsh
  at: "2026-11-19T10:18:00+09:00"
status: accepted
---

## 背景

`docs/requirements.md:140-153` の検証条件には ID が無く、
`tools/verify-web-reader.py` の 251 件の check 名との対応も無かった（R6 TD-L07）。
ADR-0090 の総数ゲートだけでは、要件から E2E 項目が消えても検出できない。
TD-L05（期待総数コメントの不整合）は既に解消済みだった。

## 決定

- `docs/requirements.md` の検証条件 8 件に VC-001〜008、
  品質要件 7 件に VC-009〜015 を付与する（元の文言は変更しない）。
- `docs/e2e-traceability.md` を追加し、
  VC ID / 要件 / E2E check 名 / 備考の対応表を記載する。
- E2E check 名は `tools/verify-web-reader.py` から AST 抽出した
  **251 件の実名のみ**を完全一致で使う。
- 未カバー要件（VC-001 フォルダ、VC-005 ZIP、VC-007 Drive、
  VC-008 OS 連携、VC-006 Tauri 実保存、VC-015 実機 5 ブラウザ等）は
  未カバー表に理由を明記する。
- 期待総数 251 はコード出典（`Checker(expected=251)`）とし、
  ADR-0090 の方針を維持する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| VC ID ＋対応表（採用） | 要件消失を手動レビューで検出できる |
| Checker に要件タグを必須化 | E2E コード変更が大きく本次範囲外 |
| 総数ゲートのみ | 形骸化リスクが残る |

## 影響

- E2E check 名・件数は変更しない（251 維持）。
- `<html>` / `_blank` を含む 3 件は markdownlint 回避のため
  トークンをバッククォートで囲むが、check 名文字列は同一。
- 1 セルに複数 check を含むため、区切りは「；」を使用。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- 親による AST 再抽出:
  - 期待 251 件／一意 251 件。
  - 対応表から抽出した 251 件と完全一致。
  - missing 0 / duplicate 0 / extra 0。
- markdownlint（requirements / traceability / index / log）: 0 issues。
- `./scripts/verify.sh`: 全ゲート成功、E2E **251 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- Checker に VC ID タグを必須化し、機械的に要件消失を検出するか。
- 未カバー要件（Drive / OS 連携 / 実機 5 ブラウザ等）の E2E 追加。
- 要件変更時に VC 対応表を自動更新する仕組み。
