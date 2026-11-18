---
type: Decision
title: "ADR-0157: インポート時の設定・履歴保護"
description: "インポートが現在の設定と履歴を破壊しないよう、現在値が存在する場合は維持する決定を記録する。"
tags:
  - ADR
  - web
  - data
generated:
  by: dsh
  at: "2026-11-18T20:45:00+09:00"
status: accepted
---

## 背景

ADR-0032 は読書位置を `savedAt` でマージする一方、
`settings` と `history` は従来どおり無条件上書きとしていた。
ADR-0151 の監査で、古いバックアップのインポートが現在の設定・履歴を
破壊し得ることが指摘された。

## 決定

- 二層構成を採用する:
  - 位置: ADR-0032 の `savedAt` マージを維持（新しい方を採用）。
  - 設定／履歴: **現在の localStorage キーが存在する場合は現在値を維持**し、
    インポート値で上書きしない。キーが無い場合のみ復元する。
- `exportedAt` は追加しない。
  - `ReaderSettings` に保存時刻が無く比較基準を新設する必要がある。
  - タイムスタンプは信頼できない入力であり、未来時刻改ざんに弱い。
  - エクスポート形式（`version: 1`）を変更すると既存 E2E の契約を壊す。
- 旧形式（`version` / `exportedAt` なし）は引き続き読み込むが、
  現在値がある場合は採用しない（後方互換は「読める」の意味）。
- 未知バージョンの fail-closed 契約は維持。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 現在値維持（採用） | 改ざん値に影響されず、形式変更も不要 |
| exportedAt 比較 | 比較基準が無く、未来時刻改ざんに弱い |
| 設定／履歴を対象外 | 既存 E2E「設定が復元される」と衝突 |

## 影響

- 現在の設定／履歴がある状態ではバックアップから復元できない
  （明示的復元オプションは未配線）。
- 位置だけは従来どおりマージされるため、
  「取り込み件数」は位置のみを数える。
- 部分設定の `false` リセット（`dark` / `followSystem` /
  `downscaleImages` の `unwrap_or(false)`）は今回対象外。
- 履歴の `savedAt` によるエントリ単位マージは今回行わず、
  キー単位で現在値を維持する。

## 検証（Runtime Proof）

- E2E に 1 件追加:
  現在設定（fontScale=4 / dark=false / downscaleImages=true / jpegQuality=50）と
  現在履歴（scrollTop=777, savedAt=2000）を置き、
  version:1 バックアップ（設定 dark=true 等、履歴 savedAt=100、
  位置 savedAt=未来）を流入させて、現在値が維持されつつ
  新規位置（scrollTop=42）だけ取り込まれることを確認。
- 期待総数 233 → **234**（check 217 + equal 17）。
- `python3 -m py_compile`、`cargo fmt --all -- --check`、wasm32 web check、
  `./scripts/verify.sh`: すべて成功、E2E **234 項目すべて成功**。
- 全単体テスト core 196 + desktop 56 + web 2、wasm-test 6、
  audit 脆弱性 0 件。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 改ざん検知（署名／チェックサム）の要否と時期。
- 設定の保存時刻を記録するメタデータの導入。
- 部分設定の `false` リセットの修正。
- 履歴のエントリ単位マージ（現在値の最大 savedAt との比較）。
- 明示的な設定／履歴復元オプションの UI 追加。
- 複数タブの localStorage 競合（既存の未決定を継続）。
