---
type: Decision
title: "ADR-0038: 初回起動時のカラースキーム追従デフォルト"
description: "設定キーが無い初回起動で OS がダークなら追従を有効にする決定を記録する。"
tags:
  - ADR
  - epub
  - web
  - wasm
generated:
  by: dsh
  at: "2026-11-17T00:05:00+09:00"
status: accepted
---

## 背景

ADR-0033 で `prefers-color-scheme` への追従を実装したが、初期値は常に
「追従しない」だった。OS がダークモードの環境では、初回起動時に読み手が
わざわざ追従ボタンを押す必要がある（ADR-0033 の未決定「追従の初期値
（OS がダークなら初回から追従を有効にするか）」）。

## 決定

### 初回デフォルト

- 設定キー（`openrustbooks:settings`）が **存在しない** 場合に限り、
  OS が `prefers-color-scheme: dark` なら `followSystem: true` を既定にする。
- OS がライトなら従来どおり `followSystem: false`。

### 後方互換（重要）

- 設定キーが **存在する** 場合は、JSON に `followSystem` が無ければ
  従来どおり `false` とし、既存ユーザーの選択を変えない。
- 既に保存された `dark` / `fontScale` / `lineHeight` の値は変えない。
- 初回デフォルトは localStorage に書き込まず、UI の状態だけに反映する。
  ユーザーが実際に設定を変更したときに初めて保存される。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 設定キーが無い場合だけ OS 設定を採用（採用） | 新規ユーザーは快適に、既存ユーザーは影響なし |
| 常に OS 設定を初期値にする | 既存ユーザーの「追従しない」選択が上書きされる |
| 初期値を変えない（現状） | ダーク環境の新規ユーザーが毎回操作が必要 |

## 影響

- OS がダークモードの新規ユーザーは、初回起動からダーク表示で読める。
- 既存ユーザーの設定・表示は一切変わらない。
- 追加依存は無く、既存の `web-sys` 機能の範囲で実装している。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 140 項目が
  5 回連続で成功。初回起動で OS がダークなら追従が有効になること、
  既存設定に `followSystem` が無い場合は追従しないこと（後方互換）を
  `Emulation.setEmulatedMedia` で確認した。
- `cargo test --all-targets --locked` は 119 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `matchMedia` の購読解除（ページ寿命まで保持しているので不要だが、
  将来 Reader が再マウントされる構成になった場合は必要）。
- `forced-colors` や `prefers-contrast` への対応。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
