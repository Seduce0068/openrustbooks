---
type: Decision
title: "ADR-0130: デスクトップ版 CSP の Wasm 対応"
description: "desktop/tauri.conf.json の script-src から unsafe-eval を除外し wasm-unsafe-eval に置換する決定を記録する。"
tags:
  - ADR
  - security
  - desktop
generated:
  by: dsh
  at: "2026-11-18T10:00:00+09:00"
status: accepted
---

## 背景

`desktop/tauri.conf.json:29` の CSP は
`script-src 'self' 'unsafe-inline' 'unsafe-eval'` で、
Wasm に必要な `'wasm-unsafe-eval'` が無かった（ADR-0091 で
Web 側には設定済み）。ADR-0128 / 0129 の未決定。

## 決定

- `desktop/tauri.conf.json:29` の `script-src` を次に変更する。
  - before: `'self' 'unsafe-inline' 'unsafe-eval'`
  - after: `'self' 'unsafe-inline' 'wasm-unsafe-eval'`
- 他のディレクティブ（`default-src` / `style-src` / `img-src` /
  `font-src` / `connect-src`）は変更しない。
  Tauri IPC (`ipc: http://ipc.localhost`) と既存送信先を維持。
- `withGlobalTauri` は今回追加しない（ADR-0128 から継続）。
- `web/index.html` の CSP は変更しない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| `'wasm-unsafe-eval'`（採用） | Tauri 公式 CSP ガイド／CSP3 の Wasm 向けトークン |
| `'unsafe-eval'` 維持 | JS の文字列 eval も許可してしまい安全でない |
| nonce/hash 化 | ADR-0091 の未決定で、Trunk ビルド変更が必要 |

## 影響

- CSP3 上 `'wasm-unsafe-eval'` は Wasm JIT を許可しつつ
  JS の `eval` を許可しない（設計意図。外部仕様の全文未取得のため
  厳密な検証は「推測」を含む）。
- meta CSP（web/index.html）と config CSP（tauri.conf.json）が
  両方注入される場合、実効は交集合になると推測される。
  双方の `script-src` は一致するため Wasm は動作する見込み。
- `style-src` / `font-src` / `connect-src` は web 側と desktop 側で
  異なる（Google Fonts / IPC）。今回の変更とは無関係の既存差分。

## 検証

- `desktop/tauri.conf.json` を `json.load` でパース成功。
- 差分は 1 行のみ（`unsafe-eval` → `wasm-unsafe-eval`）。
- `./scripts/verify.sh` rc=0（6 ゲート＋wasm-test＋audit＋E2E 229）。
- **注意**: E2E は `web/dist/index.html` の meta CSP を読むため、
  本変更（desktop 側 config）を直接検証していない。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定（csp-auditor の指摘）

- meta CSP と config CSP の優先関係／共存時の実効（推測では交集合）の実機確認。
- WebView 系統（WebView2 / WKWebView / WebKitGTK）での
  `wasm-unsafe-eval` 対応と Wasm 起動の実機確認。
- `'unsafe-inline'` 残存と nonce 未決定（ADR-0091:64）、
  `inner_html` 経路（web/src/lib.rs:3171,3194）の残リスク。
- desktop 側 CSP を直接検証する静的ゲート／`tauri build` の導入。
- 両 CSP の directive 差分の統合、`withGlobalTauri` と CSP 強化の同時実施。
