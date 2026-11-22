---
type: Decision
title: "ADR-0333: 配布保持ゲートの E2E 期待値を単一情報源化"
description: "check-distribution-path.sh が Checker(expected=N) を AST 抽出し、manifest と照合する決定。"
tags:
  - ADR
  - CI
  - distribution
generated:
  by: dsh
  at: "2026-11-23T01:20:00+09:00"
status: accepted
---

## 背景

check-distribution-path.sh は E2E expected に 256 をハードコードしていたが、
コード出典は Checker(expected=263) へ更新済みだった。
verify.sh --dist が全ゲート成功後も配布保持段階で常に失敗し、
runbook の保持・更新・ロールバック手順が実行不能だった。

## 決定

- check-distribution-path.sh が tools/verify-web-reader.py を Python AST で解析し、
  `Checker(expected=N)` の正の整数リテラルを単一情報源として抽出する。
- 抽出不能 / 複数 / 不正リテラル / 出典ファイル欠落は fail-closed。
- manifest の e2e expected を抽出版と照合し、fixture / 自己検査も同じ値を使う。
- `passed: 0` は「1 件も実行していない」成功風記録として拒否する。
- self_test の成功ログを stderr へ出し、stdout は正規化 DIST_ROOT のみに限定する。
- verify.sh の DIST_ROOT 解決は `--no-self-test` 付きで呼ぶ。
- runbook-verified-build / runbook-release の現行値を 263 へ更新し、
  253 / 255 / 256 は履歴値として維持する。

## 検証（Runtime Proof）

- 負向 6 ケース + 出典欠落（code=264 / manifest=263 不一致 / expected kw 欠落 /
  文字列リテラル / Checker 2 件 / 出典欠落）すべて期待 rc。
- bash -n / shellcheck / shfmt -d: rc=0。
- markdownlint: 0 issues。
- `./scripts/verify.sh`: 全ゲート成功、E2E **263 passed / 0 failed**、WDIO gate rc=0。
- `./scripts/verify.sh --dist /home/koppe/.local/share/openrustbooks-verified`:
  rc=0、`検証済み dist を保持しました: .../ef36be0-20261122T162513Z`。

## 未決定

- verify.sh の grep 抽出を共通 AST ヘルパーへ集約するか（現状は checker 側が AST、manifest 生成側は grep）。
