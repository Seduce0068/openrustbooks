---
type: Decision
title: "ADR-0280: Tauri plugin config の無効キー削除"
description: "tauri.conf.json の plugins.*.scope.allowedPaths が無効キーであるため、plugins ブロックを削除し再混入を静的ゲートで防ぐ決定を記録する。"
tags:
  - ADR
  - tauri
  - config
generated:
  by: dsh
  at: "2026-11-21T02:27:00+09:00"
status: accepted
---

## 背景

`desktop/tauri.conf.json` は `plugins.fs/dialog/shell.scope.allowedPaths` を保持していた。
Tauri 2.12.5 の fs / shell plugin config は `deny_unknown_fields` で未知キーを拒否し、
dialog には config 型が無い。
`desktop/gen/schemas/desktop-schema.json` は CapabilityFile 用で config 検証には使えず、
公式 config schema も PluginConfig を不透明に扱うため、 IDE 上の検証では無効キーを検出できない。
capability ファイルは存在せず、実効スコープは元から空だった。

## 決定

- `desktop/tauri.conf.json` から `plugins` ブロック全体を削除する。
- `app` / `bundle` / `app.security.csp` / plugin 初期化 / 独自 command 登録は維持する。
- `scripts/check-tauri-plugin-config.sh` を追加し、次を fail-closed で検査する:
  - トップレベルに `plugins` が無いこと
  - JSON キーとして `scope` / `allowedPaths` が存在しないこと
  - `desktop/src/main.rs` に fs / dialog / shell の初期化が残ること
  - JSON 欠落 / 不正 JSON
- `verify.sh` と `ci.yml` の `extra_gates` から同じ script を呼ぶ。
- capability ファイルは追加しない（ADR-0278 の判断を維持）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| plugins ブロック削除と静的ゲート（採用） | 無効キーを除き、再混入を防げる |
| `scope.allow` などへ推測置換 | pin version で無効な可能性があり採用しない |
| capability ファイルを追加 | 実効スコープは元から空で、今回の最小修正ではない |
| 現状維持 | plugin 初期化時の deserialize 失敗リスクが残る |

## 影響

- 実効権限は狭まらない。スコープは元から設定されておらず、
  設定キー自体が未適用だった。
- 「スコープを削除して安全になった」という記録はしない。
- 静的ゲートは設定テキストのみを検査し、
  OS WebView の描画 / IPC 注入 / CSP 実効 / a11y は証明しない。
- E2E check 名・件数（256）は不変。

## 検証（Runtime Proof）

- `bash scripts/check-tauri-plugin-config.sh`: rc=0。
- 負向 test: plugins 復活 / `scope` + `allowedPaths` キー復活 /
  tauri.conf.json 欠落 / 不正 JSON はすべて rc=1。
- 有効な JSON の説明文中にある "scope" という語は rc=0 で誤検知しない。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 実 Tauri WebView の起動挙動（ADR-0273 の環境ブロッカー）。
- fs / shell plugin が本当に必要か
  （Rust 側で使われるのは dialog の `DialogExt` のみ）。
- 将来 capability を追加する場合の permission と
  `$DOCUMENT/*` / `$DOWNLOAD/*` の glob 意味（ADR-0278 の supersede 要否）。
- config schema の vendoring と pin version 2.12.5 /
  公式 schema `$id` 2.12.7 の整合。
