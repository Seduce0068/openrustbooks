---
type: Decision
title: "ADR-0094: Desktop メインウィンドウ取得の Result 化"
description: "get_webview_window の unwrap() を Result 化し、起動時の panic を防ぐ決定を記録する。"
tags:
  - ADR
  - desktop
  - bugfix
generated:
  by: dsh
  at: "2026-11-18T01:20:00+09:00"
status: accepted
---

## 背景

`build_app().setup()` 内の `app.get_webview_window("main").unwrap()` は、
メインウィンドウが見つからない場合にパニックしていた（todo.md C05 の一部、
code-scanner #10）。

## 決定

- `unwrap()` を `ok_or_else(...)?` に置き換え、
  `std::io::ErrorKind::NotFound` を `Box<dyn Error>` として返す。
- `setup_window` と AI 初期化の挙動は変更しない。
- Web 版には影響しない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| Result 化（採用） | Tauri のセットアップ失敗として扱え、panic より安全 |
| `expect` に置換 | 依然として panic する |
| ウィンドウ生成を retry | 原因が分かりにくく、今回の問題とは別 |

## 影響

- メインウィンドウが無い場合、アプリ起動がエラーとして終了する（panic しない）。
- 通常起動の挙動は変わらない。

## 検証

- `cargo check --all-targets --locked` が成功。
- `./scripts/verify.sh` を実行し、6 ゲート・trunk build・WASM パス検査・
  E2E 223 項目がすべて成功した。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `.run()` の `expect("error while running tauri application")` の扱い。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
