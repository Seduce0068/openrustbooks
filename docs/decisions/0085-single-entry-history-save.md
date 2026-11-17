---
type: Decision
title: "ADR-0085: 履歴 1 件の localStorage 保存"
description: "履歴が 1 件になっても localStorage を更新する決定を記録する。"
tags:
  - ADR
  - web
  - bugfix
generated:
  by: dsh
  at: "2026-11-17T21:40:00+09:00"
status: accepted
---

## 背景

`save_history()` は `history.len() <= 1` のとき早期リターンしていた。
そのため履歴クリア（ADR-0031）や個別削除（ADR-0035）で 1 件になったとき、
localStorage には古い履歴が残り、再読み込みで古い履歴が復活していた
（todo.md B09）。

## 決定

- `save_history()` の早期リターンを削除し、履歴が 1 件のときも
  `{ entries: [現在位置], index: 0 }` を保存する。
- `load_history()` は従来どおり 2 件以上のみ復元する。
  1 件の保存データは無視され、次回の現在位置から新しい履歴が始まる。
- E2E で、クリア後に「entries が 1 件で index が 0 の履歴キー」が
  存在することを確認する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 1 件でも保存（採用） | ADR-0031 の「現在位置のみの履歴を保存」という決定に合致 |
| キーを削除 | 現在位置の記録が消え、復元時の起点が失われる |
| 早期リターンを維持 | 古い履歴が復活するバグが残る |

## 影響

- クリア・個別削除後も localStorage に古い履歴が残らない。
- 再読み込み時に履歴が 1 件（現在位置のみ）から始まる。
- 既存の 125 テスト、E2E は新規チェック 1 件を加えて通る。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 220 項目が
  5 回連続で成功。
- デバッグで、アプリ本体の履歴キーが 1 件に更新されることを確認した。
  古いテスト用キーが残っていたため、E2E の判定は「いずれかの履歴キーが
  1 件」に変更した。
- `cargo test --all-targets --locked` は 125 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 古いテスト用キーの削除（E2E ハーネス側の都合で残る）。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
