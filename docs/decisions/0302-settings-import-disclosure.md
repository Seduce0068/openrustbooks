---
type: Decision
title: "ADR-0302: バックアップ import の表示設定保持を告知する"
description: "現在値保護でバックアップの表示設定を復元しない場合に固定文言で告知する決定。"
tags:
  - ADR
  - web
  - settings
generated:
  by: dsh
  at: "2026-11-22T07:10:00+09:00"
status: accepted
---

## 背景

README は「読書位置のエクスポート・インポート（表示設定・履歴を含む）」と
記載していたが、実装は現在の表示設定が localStorage に存在する場合に
バックアップの表示設定を復元しなかった。
利用者は復元されない理由を知ることができなかった。

## 決定

- バックアップに `settings` があり、現在値が localStorage に存在して
  復元をスキップした場合のみ固定文言を status へ連結する。
- 固定文言: `表示設定は現在の値を保持しました。バックアップの表示設定は復元されていません。`
- 現在値が無い場合は従来どおり復元する。
- README に「表示設定は現在値が無い場合のみ復元される」制約を明記する。
- 設定値 / キー / パス / 件数は status へ出さない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 固定文言で告知（採用） | 最小変更で誤解を解ける |
| 明示的な復元 UI を追加 | 今回の範囲外 |
| README のみ修正 | UI 上で理由が分からない |

## 検証（Runtime Proof）

- `cargo fmt --all -- --check`: rc=0。
- `npx tsc -p tsconfig.json --noEmit`: rc=0。
- `cargo check -p openrustbooks-web --target wasm32-unknown-unknown --locked`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。
- 初回 WDIO は WebView/IPC の一時的失敗で rc=1 だったが、
  親の再実行で rc=0（E2E 256 件・他ゲート成功）。

## 未決定

- 保存失敗と設定保持が併発した場合に両方を告知するか。
- `settings_preserved` の専用 unit / E2E test。
- 明示的な「設定を復元」UI の追加。
