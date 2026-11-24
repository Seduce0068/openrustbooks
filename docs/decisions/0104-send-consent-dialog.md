---
type: Decision
title: "ADR-0104: AI 送信前のネイティブ同意ダイアログ"
description: "本文を AI へ送る直前に Rust 側でネイティブ確認を要求する決定を記録する。"
tags:
  - ADR
  - security
  - ai
  - desktop
generated:
  by: dsh
  at: "2026-11-18T03:40:00+09:00"
status: accepted
---

## 背景

AI は既定無効（ADR-0099）、service は無効時にガード（ADR-0100）、送信先は
許可リスト制限（ADR-0101）されている。しかし利用者が AI を有効にした後は、
どの本文が送られるかを確認せずに送信できた（todo.md D06 の本体、
ADR-0099 〜 0103 の未決定）。

## 決定

- 本文を送る 4 コマンド（`generate_markdown` / `refine_markdown` /
  `convert_to_table` / `generate_front_matter`）に `AppHandle` を追加し、
  送信直前に Rust 側でネイティブ確認ダイアログを表示する。
- JS から `consent: bool` は受け取らない。侵害された renderer が
  `true` を渡せるため、同意の証明にならない。
- `tauri-plugin-dialog` の `app.dialog().message(...).show()` を
  `tokio::sync::oneshot` で await する（`blocking_show()` は使わない）。
- ダイアログ本文には操作名と、送信字段ごとの「文字数＋先頭 80 文字」を
  並べた要約を表示する。本文全体と API キーは出さない。
- `Ok(true)` のみ送信を続行し、`Ok(false)`（キャンセル）とチャネル閉鎖は
  いずれも Err を返す。
- `lint_markdown` / `lint_and_fix_markdown` はネットワーク送信がないため
  ダイアログを出さない。
- Tauri の command 登録（`main.rs`）は変更しない
  （`AppHandle` は自動注入で、JS 側の invoke 引数は不変）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| Rust 側ネイティブダイアログ（採用） | JS の bool より強い同意証明になる |
| JS から `consent: bool` | 侵害された renderer が常に true を渡せる |
| 警告ログのみ | 送信を止められない |

## 影響

- AI 操作のたびに OS の確認ダイアログが出る（UX 上の摩擦は意図的）。
- ダイアログ要約には本文の先頭 80 文字が含まれるため、
  機微情報が画面に一時表示される（全文は出さない）。
- 新規依存は無く、IPC 引数も変わらない。

## 検証

- 追加テスト 3 件（要約の内容、マルチバイト 80 文字切り詰め、
  空 field の 0 chars 表示）。
- `cargo test --all-targets --locked`: core 148 + desktop 21、すべて成功。
- `./scripts/verify.sh` を実行し、6 ゲート・trunk build・WASM パス検査・
  E2E 228 項目がすべて成功した。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 全文プレビューを別 UI（スクロール領域付き）で見せる機能。
  `render()` の生出力には Handlebars 制御構文が残るため、整形すると
  実際の送信内容と不一致になる（preview-designer の指摘）。
- OpenAI の固定 system message をプレビューに含めるか。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
