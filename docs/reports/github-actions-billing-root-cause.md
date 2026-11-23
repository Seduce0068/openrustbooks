---
type: Reference
title: "GitHub Actions billing Failure の原因調査"
description: "Seduce0068/openrustbooks の CI が billing で失敗する原因を一次情報と実ログから切り分けた調査レポート。"
tags:
  - GitHub Actions
  - billing
  - investigation
generated:
  by: dsh
  at: "2026-11-23T13:10:00+09:00"
status: accepted
---

## 調査範囲

Seduce0068/openrustbooks の CI run（2026-11-15〜2026-11-22）と、
呼び出し先 workflow、同アカウントの他リポジトリの run を読み取り専用で調査した。
支払い設定・spending limit・認証スコープの変更は行っていない。

## 確認済み事実

- 失敗 run 45061410838 は 2026-11-22T19:33:19Z に開始し 3 秒で完了、
  job `gates / gates` の steps は空だった。
- check-run annotation の原文:
  "The job was not started because recent account payments have failed or your spending limit needs to be increased.
  Please check the 'Billing & plans' section in your settings"
  — [check-runs annotations API][annotations]
- 同一 annotation は Seduce0068 の他の private リポジトリ（agents / jarvis / openIllustration）でも確認され、アカウント単位のブロックである。
- 同時期に public リポジトリ（cordis-rs / gpui-file-explorer）では同じ `gates / gates` が開始・成功しており、ランナー枯渇や共通 workflow の参照権限拒否は否定的。
- 最後の成功 run は 2026-11-15T04:48:13Z（43678335690）、最初の失敗 run は 2026-11-15T06:37:16Z（43686274863）。この間 billing ブロックが始まった。
- 境界前後で `.github/workflows/ci.yml` と `Seduce0068/workflows/rust-gates.yml@v1`（sha `b7675efd...`）は同一で、設定変更が原因ではない。
- `gh auth status` の token scopes は `gist`, `read:org`, `repo`, `workflow` で `user` が無く、
  `GET /users/Seduce0068/settings/billing/actions` は scope 不足で照会できない。

## 未確認

- 「支払い失敗」と「spending limit 到達」のどちらか、および使用量・上限の具体的数値。
- GitHub Web の Billing & plans 画面での目視確認（トークンではセッション認証不可）。

## 結論

原因は openrustbooks のコードや workflow ではなく、
GitHub アカウントの請求状態による private repository の Actions 実行拒否である。
CI 設定を変更しても解消しない。

## 次の一手

1. GitHub Web で Settings > Billing & plans を開き、支払い方法と spending limit を確認する。
2. 必要なら `gh auth refresh -h github.com -s user`（認証情報変更を伴うため未実施）後に
   `/users/Seduce0068/settings/billing/actions` と `/usage` で数値を照合する。
3. 解消後に run を再実行し、`gates / gates` の steps が 18 個実行されることを確認する。

[annotations]: https://api.github.com/repos/Seduce0068/openrustbooks/check-runs/137327936288/annotations
