---
type: Decision
title: "ADR-0159: CLS の E2E 直接計測"
description: "PerformanceObserver('layout-shift') で CLS を計測し、画像章オープン時のレイアウトシフトをゲートする決定を記録する。"
tags:
  - ADR
  - testing
  - web
generated:
  by: dsh
  at: "2026-11-18T21:30:00+09:00"
status: accepted
---

## 背景

ADR-0149 で画像の `width` / `height` 予約を実装したが、
「CLS が減った」ことを直接検証するチェックが無かった
（ADR-0136 / 0149 の未決定）。

## 決定

- `tools/verify-web-reader.py` に `CLS_PROBE` を追加し、
  `Page.addScriptToEvaluateOnNewDocument` で BOOT_PROBE 直後に登録する。
  - `PerformanceObserver('layout-shift')`、`buffered: true`。
  - `entry.hadRecentInput` は入力由来のため除外。
  - `window.__orbCls` / `__orbClsEntries` へ蓄積。
  - `__orbClsReady` / `__orbClsError` で observer 生存を判定。
- チェック「画像読み込み後の CLS が閾値以下に収まる」を
  位置復元後の画像セトリング直後・表示設定変更前に追加。
  - `tall_loaded`、observer ready、error 無し、CLS が number、
    `CLS <= CLS_THRESHOLD` を確認。
- 閾値は **0.1**（Web Vitals の "good" 境界）。
- 計測窓は `Page.reload` 直後の同一文書・画像セトリングまでの数秒に限定し、
  単純合計でもセッションウィンドウ CLS と同値とみなす。
- 期待総数を 234 → **235**（check 218 + equal 17）に更新。
- 本番コードは変更しない。

## 実測（Runtime Proof）

- 一時スクリプトで 5 回連続計測:
  - 初回第 2 章（縦長画像読込後）: `[0, 0, 0, 0, 0]`、entries 0 件。
  - 位置復元後・画像セトリング後（E2E と同一地点）: `[0, 0, 0, 0, 0]`、
    entries 0 件、`#tall` の naturalHeight=4000 / width,height=(3000,4000)。
- 陽性対照（意図的レイアウトシフト注入）: 0 → **0.106** を捕捉、
  observer 動作を確認（E2E 本体には含めない）。

## 検出力の限界（重要）

- `width` / `height` 属性を除去する宣言的リグレッションを再現したところ、
  CLS は 5 回とも 0 で閾値を超えなかった。
- 理由: `images_settled()`（最大 2 秒）が画像読み込み完了まで
  本文 DOM 挿入を遅らせるため、属性が無くてもシフトが発生しない。
- したがって本チェックは「画像章オープン時のレイアウトシフト全般」の
  ガードであり、**寸法属性欠落の直接検知器ではない**。
  寸法属性の検証は ADR-0149 の E2E（`width` / `height` の一致）が担う。

## 検証

- `python3 -m py_compile`、fresh trunk build、`./scripts/verify.sh`: 成功。
- E2E **5 回連続すべて 235/0**、CLS チェックも全回 ok。
- 全単体テスト core 196 + desktop 56 + web 2、wasm-test 6、
  audit 脆弱性 0 件。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 5 ブラウザ実機検証（Chrome 157 headless のみで実施）。
  Firefox / Safari は LayoutShift 非対応の可能性。
- セッションウィンドウ方式の完全実装（現状は狭い窓の単純合計）。
- `IMAGE_SETTLE_MS = 2000` の根拠、上限 16384px の根拠。
- 縮小後属性の微シフト、`.reader-cover` の対象範囲、
  作者指定 `width` / `height` の上書き。
