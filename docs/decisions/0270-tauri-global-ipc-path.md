---
type: Decision
title: "ADR-0270: Tauri グローバル IPC 経路の選択"
description: "withGlobalTauri を有効化し、web の __TAURI__.core.invoke 参照を公式グローバル経路へ統一する決定。"
tags:
  - ADR
  - tauri
  - ipc
generated:
  by: dsh
  at: "2026-11-20T05:13:00+09:00"
status: accepted
---

## 背景

ADR-0264 は IPC 経路を未決定として残していた。
`web/src/lib.rs` の invoke ラッパーは `window.__TAURI.core.invoke` を参照していたが、
Tauri 2.12.5 の `bundle.global.js` が公開するのは
`window.__TAURI__.core.invoke`（末尾 `__` 付き）で、
既定の `withGlobalTauri: false` では注入されない。
このため desktop でも `Err("desktop only")` になる可能性が高かった。

## 決定

- `desktop/tauri.conf.json` の `app` 直下に `withGlobalTauri: true` を追加する。
- `web/src/lib.rs` の参照を `__TAURI__` に修正し、
  Reflect で `__TAURI__` → `core` → `invoke` を辿る経路に統一する。
- CSP 値は変更しない（`script-src` は既に `'unsafe-inline'` を含み、
  Trunk inline script と Tauri 注入 script の双方を許可）。
- `__TAURI_INTERNALS__` 直接利用と npm `@tauri-apps/api` は採用しない。
- browser 単体の fail-closed（`__TAURI__` が無ければ `Err("desktop only")`）を維持する。
- 静的 CSP ゲートに `withGlobalTauri: true`、IPC reflect 経路、
  `desktop only` 固定文言の残存を検査させる。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| withGlobalTauri + `__TAURI__.core.invoke`（採用） | 新規依存なし・CSP 緩和不要・公式グローバル契約 |
| npm `@tauri-apps/api` | 公式推奨だが bundler / package-lock / オフライン再現性の追加が必要 |
| `__TAURI_INTERNALS__` 直接 | 最小差分だが非公式で upgrade 契約が無い |
| 現状維持 | desktop の設定読み取りすら動作しない可能性が残る |

一次情報:
- Tauri 2.12.5 `bundle.global.js`: `window.__TAURI__=__TAURI_IIFE__;`
  および `return e.app=..., e.core=g, ...`（`g.invoke` は internals を呼ぶ）。
- Tauri 公式「Calling Rust from the Frontend」:
  `const invoke = window.__TAURI__.core.invoke;`
- Cargo.lock の tauri 実バージョン: 2.12.5。

## 影響

- desktop で `get_config` の読み取り経路が有効になる（実 WebView 未検証）。
- browser E2E は `__TAURI__` 不在の fail-closed を引き続き検証する。
- E2E check 名・件数（255）は不変。
- 外部 JS 利用者の有無は未決定のまま。

## 検証（Runtime Proof）

- `bash -n` / `shellcheck` / `python3 -m json.tool`: OK。
- 静的ゲート負向 test: withGlobalTauri false / 欠落 / 非 bool、
  `__TAURI__` 参照欠落、末尾欠落 `__TAURI`、`core` / `invoke` 参照欠落、
  `desktop only` 欠落 — すべて rc=1。
- `./scripts/verify.sh`: csp-consistency rc=0、全ゲート成功、
  E2E **255 passed / 0 failed**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 実 Tauri WebView での往路検証（ADR-0253 実装前提 6、ローカル CDP では不可）。
- nonce/hash 化と meta/config CSP 一本化（ADR-0091 / 0130 / 0264 / 0268）。
- Tauri 更新時に `__TAURI__` / `core.invoke` 形状が変わり得ることの検知手順。
- 外部 JS 利用者の有無（破壊的変更の扱い）。
