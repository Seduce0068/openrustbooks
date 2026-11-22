---
type: Decision
title: "ADR-0310: runbook-release / README の現行状態追従"
description: "updater 未登録、実 WebView 検証済み、build.rs 配置、ゲート一覧、ADR 要約を現行実装へ追従させる決定。"
tags:
  - ADR
  - documentation
  - release
generated:
  by: dsh
  at: "2026-11-22T10:06:00+09:00"
status: accepted
---

## 背景

runbook-release は updater を「初期化のみ」と記載していたが、
ADR-0282 で plugin 登録と依存が削除されていた。
README は build.rs の配置誤記、ADR 要約の欠落 / 誤配置、
ゲート一覧の古さが残っていた。

## 決定

- updater は「登録なし / `.check()` なし / 固定表示 / check 禁止ゲート」と記載する。
- 設定保存は実 WebView 検証済み（ADR-0282）と WDIO ゲート（ADR-0283）を反映し、
  ADR-0273 / 0274 は当時の履歴として扱う。
- README の tree を `desktop/build.rs`（クレート直下）へ修正する。
- `--locked` は未決定ではなく現行契約として記載する。
- ADR 要約の欠落行を補完し、誤配置を各 ADR 行へ戻す。
- ゲート一覧に dependency-hygiene / audit-db-freshness / build-desktop /
  check-frontend-dist / WDIO を追加する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 文書を現行実装へ追従（採用） | 利用者と運用者の誤認を防ぐ |
| ADR のみ更新 | runbook / README が古いままになる |
| 公開鍵 / endpoint を追加 | 未実装外出力であり却下 |

## 検証（Runtime Proof）

- markdownlint: 0 issues。
- R4 再レビュー: 初期 high 2 件 + medium 3 件すべて反映済み。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- 公開鍵 / endpoint / Release workflow / 署名成果物（ADR-0222 / 0254）。
- 実機 5 ブラウザ検証（Chrome / Edge は当時 255/0、他は環境ブロッカー）。
- CI billing 解消後の実 CI ゲート検証（CI-H01）。
