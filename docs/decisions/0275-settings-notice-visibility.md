---
type: Decision
title: "ADR-0275: ブラウザ専用設定通知の表示範囲"
description: "desktop で設定保存 UI が有効なとき、ブラウザ専用の読み取り説明を DOM から除外する決定を記録する。"
tags:
  - ADR
  - web
  - a11y
generated:
  by: dsh
  at: "2026-11-20T06:52:00+09:00"
status: accepted
---

## 背景

ADR-0193 / 0253 は、ブラウザ版の設定フォームが読み取り専用であることを
利用者へ常に説明すると定めていた。
ADR-0272 で desktop の設定保存 UI が接続された結果、
desktop で編集可能なのに「ブラウザ版は表示のみです…」が常時表示され、
誤解を与えるようになった。

## 決定

- 当該説明は `editable = false`（ブラウザ、または IPC 読込失敗時）のときのみ表示する。
- `editable = true`（desktop で `get_config` 成功）のときは
  Leptos `<Show>` で子ノードを DOM から除外する（CSS 非表示ではない）。
- `role="status" aria-live="polite"` の status 要素は常時維持する。
- IPC 経路、構造化エラー、DTO、a11y 属性、check 名・件数は変更しない。
- 既存 Tauri mock check に、desktop で当該説明が DOM に無いことを
  AND 条件として追加する（check 数は不変、256 維持）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| browser のみ条件表示（採用） | 誤解を解消しつつ既存 browser E2E 2 件を維持できる |
| 常時表示のまま | desktop で編集可能なのに「保存不可」と表示され誤解を与える |
| 文言を desktop 向けに変更 | 既存 E2E / ADR-0193 の契約を広範囲に変更する必要がある |

## 影響

- browser: 従来どおり説明が表示され、入力は readonly / disabled。
- desktop: IPC 成功後に説明が DOM から消え、保存ボタンが有効。
- E2E check 名・件数は 256（check 239 + equal 17）のまま。
- 実 WebView の表示は未検証（ADR-0273）。

## 検証（Runtime Proof）

- `cargo test --all-targets --locked`: core 287 / desktop 82 / web 2。
- `wasm-pack test --node --locked`: 14 passed。
- browser E2E 2 件（fallback / readonly）は不変。
- Tauri mock check は成功文言、revision +1、JS エラー 0、
  ブラウザ専用説明の非存在を検証。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 実 Tauri WebView での条件表示と a11y（ADR-0273 の検証手順へ追加する）。
- 初期描画時に `editable=false` の説明が一瞬現れることの
  スクリーンリーダー影響（実機確認が必要、推測）。
- ADR-0193 / 0253 の「常時説明」条項を履歴として扱う範囲
  （本 ADR が desktop 側の契約を更新）。
