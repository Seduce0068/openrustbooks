---
type: Decision
title: "ADR-0030: 履歴のエクスポート・インポート"
description: "エクスポート・インポート形式に移動履歴を含める決定を記録する。"
tags:
  - ADR
  - epub
  - web
  - wasm
generated:
  by: dsh
  at: "2026-11-15T18:32:00+09:00"
status: accepted
---

## 背景

ADR-0026 まででエクスポート形式は `{ positions, settings }`。履歴は
ADR-0024 で localStorage に永続化されているが、バックアップの対象に
含まれていなかった（ADR-0026・ADR-0029 の未決定「履歴のエクスポート・
インポートへの含有」）。端末をまたいで読書を続けるには、履歴も
バックアップの対象が望ましい。

## 決定

### エクスポート

- 出力 JSON に `history` を追加する。
  `{ "positions": {...}, "settings": {...}, "history": { "<履歴キー>": { "entries": [...], "index": n }, ... } }`
- `history` には `openrustbooks:history:` で始まる全キーを含める。
- 値は既存の `SavedHistory`（entries / index）と同じ形にする。
- localStorage から読んだ値が `SavedHistory` として読めない場合は
  含めない（fail-closed）。

### インポート

- `history` キーがあれば各履歴を取り込む。
- 値が `SavedHistory` として読めるものだけ localStorage に書き込む
  （fail-closed）。
- `history` が無い形式（旧形式）でも従来どおり positions / settings だけ
  取り込む。
- 状態メッセージは `Imported N reading positions and M histories.` の形で
  位置と履歴の両方の件数を報告する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 同一ファイルに `history` を追加（採用） | バックアップが 1 ファイルで済む |
| 履歴だけ別ファイルにする | ファイルが増え扱いが悪い |
| 履歴はバックアップしない | 端末をまたいで読書を続けられない |

## 影響

- 履歴もバックアップ・復元の対象になる。
- 旧形式（history 無し）のバックアップも引き続きインポートできる。
- インポートの状態メッセージの文言が変わる（件数が 1 つ増える）。
- 追加依存は無く、既存の `web-sys` 機能の範囲で実装している。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 116 項目が
  3 回連続で成功。履歴について、エクスポート対象の履歴が保存されている
  こと、新形式のインポートで履歴が復元されることを確認した。
- `cargo test --all-targets --locked` は 119 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 履歴エントリの個別削除・全消去（ADR-0029 の未決定）。
- インポート時のマージ（上書きではなく統合）（ADR-0026 の未決定）。
- エクスポート形式のバージョニング。
- クラウド連携時の自動同期（ADR-0015 から継続）。
