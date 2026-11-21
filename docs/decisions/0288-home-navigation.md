---
type: Decision
title: "ADR-0288: ホーム画面の主要機能導線と未接続 UI の明示"
description: "Reader / Editor への導線を追加し、未接続ボタンとダミー FileTree を誤解のない表示へ変更する決定。"
tags:
  - ADR
  - UX
  - web
generated:
  by: dsh
  at: "2026-11-22T03:18:00+09:00"
status: accepted
---

## 背景

HomePage の nav は Settings / About のみで、Reader / Editor は URL 直入力が必要だった。
New File / Open Folder / Open File は on:click が無いのにクリック可能に見え、
FileTree は固定ダミー項目を表示していた。

## 決定

- HomePage の nav に Reader / Editor への導線を追加する。
- リンク先は `/reader/sample` / `/editor/sample` とし、
  サンプル文書であることを文言と title / aria-label で明示する。
- 未接続の New File / Open Folder / Open File は disabled にし、
  理由を title / aria-label で示す。
- FileTree の固定ダミー項目を削除し、
  「ファイル一覧は未接続です」の固定表示へ置き換える。
- `.file-tree li` の `cursor: pointer` と hover スタイルを削除する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| サンプル導線 + 未接続明示（採用） | 既存ルートを壊さず主要機能へ到達できる |
| ファイル選択 UI を実装 | 今回の範囲外 |
| E2E 用 ID `/e2e` を本番 UI に残す | 利用者に内部用 ID が露出するため却下 |

## 影響

- E2E が直接開く `/reader/e2e` / `/editor/e2e` は維持する。
- Settings / About の導線と各 route は変更しない。
- 未接続ボタンは disabled のためキーボードではフォーカスされない。
  理由を可視化する場合は今後の課題。
- 主要ナビの英語文言と `<nav>` のランドマーク名は R4 の低優先指摘として残す。

## 検証（Runtime Proof）

- `cargo fmt --all -- --check`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- 実ファイル選択 UI の実装。
- 未接続理由の可視テキスト化（`aria-disabled` の検討）。
- ナビ / ボタン文言の日本語統一と `<nav aria-label>` の付与。
