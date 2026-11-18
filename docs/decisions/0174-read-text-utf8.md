---
type: Decision
title: "ADR-0174: read_text の不正 UTF-8 扱い"
description: "不正 UTF-8 の内容をエラー文言へ埋め込まず、U+FFFD 置換で返す決定を記録する。"
tags:
  - ADR
  - core
  - security
generated:
  by: dsh
  at: "2026-11-19T02:10:00+09:00"
status: accepted
---

## 背景

`LocalStorage::read_text()` は不正 UTF-8 のとき
`String::from_utf8_lossy(e.as_bytes())` の結果（ファイル内容の断片）を
`CoreError::Io` の文言へ埋め込んでいた。
Tauri IPC 経由で frontend へ返る／ログへ出ると意図しない文書内容が漏れる
（R6: PR-H02 / TD-H02）。コメントは lossy フォールバックを謳っていたが、
実際はエラーを返していた。

## 決定

- `read_text()` は `match String::from_utf8(bytes)` で実装する。
  - 有効 UTF-8: バッファを再利用してそのまま返す（ゼロコピー）。
  - 不正 UTF-8: `String::from_utf8_lossy(error.as_bytes()).into_owned()` で
    U+FFFD に置換して返す。
- どちらの経路でもエラーを生成しない。
  エラー文言へ内容を載せる経路を完全に廃止する。
- トレイト doc と `ReadOptions::as_text()` のコメントを
  「UTF-8 として読み、不正列は U+FFFD に置換」へ修正。
- ZIP / EPUB の `String::from_utf8` fail-closed（ADR-0004）は変更しない
  （本次は `read_text()` のみ）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| lossy 置換して `Ok`（採用） | コメントどおりで漏洩経路も消える |
| 理由のみのエラー | 文書が読めず UX が劣る |
| 内容入りエラー | 漏洩のため却下 |

## 影響

- `read_text()` の意味が「不正ならエラー」から「置換して成功」へ変化。
  プロダクション呼び出し側は現状存在しない（監査で確認）。
- 将来の呼び出し側が corrupt 入力をサイレント受理し、
  編集→保存で元バイトが失われる可能性は **推測**（未決定）。
- 巨大ファイルのピークメモリは有効 UTF-8 では増えない（ゼロコピー維持）。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- 新規テスト 2 件:
  - `[b'A', 0xff, b'B']` → `assert_eq!(text, "A\u{FFFD}B")`、
    エラー文言非含有。
  - 有効 UTF-8「こんにちは」がそのまま返る回帰。
- `cargo test --all-targets --locked`: core **210** + desktop 56 + web 2。
- wasm-test 10、audit 脆弱性 0 件。
- `./scripts/verify.sh` rc=0、E2E **246 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 将来 `read_text()` を編集フローへ接続するときの
  corrupt 入力の扱い（置換保持 vs 保存拒否）。
- EPUB / ZIP の fail-closed 方針との UX 統合。
