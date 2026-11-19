---
type: Decision
title: "ADR-0264: Tauri IPC 経路の設計受容"
description: "withGlobalTauri 未設定で desktop も IPC が動かない可能性を記録し、3 経路を比較してコード変更なしの設計受容を選択する。"
tags:
  - ADR
  - tauri
  - ipc
  - csp
generated:
  by: dsh
  at: "2026-11-20T03:20:38+09:00"
status: accepted
---

## 背景

R6 A05 第一段の設計補強として、Tauri IPC の現状と経路を記録する（swarm-coding / design）。
現状は次のとおりで、desktop も IPC が機能していない可能性がある。

- `web/src/lib.rs:4096-4121` の `invoke()` は
  `window.__TAURI.core.invoke` を `Reflect::get` で辿り、
  無い場合はすべて `Err("desktop only")` を返す。
  `fetch_config()` はこの `invoke()` 経由で `get_config` を呼ぶ。
- `desktop/tauri.conf.json` に `withGlobalTauri` は無い。
  Tauri 公式の "Calling Rust from the Frontend" は、
  グローバルスクリプト（`window.__TAURI__.core.invoke`）を使うには
  `tauri.conf.json` の `app.withGlobalTauri` を `true` に設定するよう求める。
  したがって desktop の WebView でも `window.__TAURI` が注入されず、
  `fetch_config()` が `Err("desktop only")` になる可能性がある。
  **実 Tauri WebView での往路検証は未実施で、断定しない。**
- 同設定の `app.security.csp` は
  `script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval'` を含み、
  CSP の強化方針は ADR-0091 / 0130 / 0191 の対象と隣接する。
- ADR-0253 は desktop の保存 UI を段階設計と定め、
  実装前提 1 で `withGlobalTauri` と CSP の方針を確定させる前提にしている。
  本 ADR はその前提 1 の判断材料を補強するもので、実装ではない。

## 決定

- コード変更なしで現状を設計受容する。
  `web/src/lib.rs` / `desktop/tauri.conf.json` / CSP / capability / E2E は変更しない。
- 経路の選択は [ADR-0270](0270-tauri-global-ipc-path.md) で決定した
  （`withGlobalTauri: true` + `__TAURI__.core.invoke`）。
  実装は ADR-0253 の段階設計に従って進める。
- browser 単体の読み取り専用 / fail-closed 契約（ADR-0193 / 0253）は維持する。
  desktop が未接続の可能性がある間も、誤って保存可能と表示しない。

## 選択肢の比較

| 経路 | 利点 | 追加コスト / リスク | 判断 |
| --- | --- | --- | --- |
| npm `@tauri-apps/api` | 公式推奨の `invoke` import。型付き API で契約が明確 | JS bundler と `package-lock.json` の追加、オフライン再現性の担保が必要。web には npm が無く Trunk のみで、ビルドと CI の前提を変える | 今回は採らない（前提変更を伴う） |
| `__TAURI_INTERNALS__` 直接 | 最小差分。内部的に注入されるため他有無に依存しにくい（非公式情報・実機未検証） | 非公式の内部 API で upgrade 契約が無い。Tauri 側の変更で壊れるリスクがあり、静的ゲートでも検知しにくい | 今回は採らない（upgrade リスクを未決定に残す） |
| `withGlobalTauri: true` | 公式のグローバル経路。既存の `window.__TAURI` 参照に近い | CSP の nonce / hash と Trunk が生成する inline script の扱いが未確定。`'unsafe-inline'` の削除方針と同時設計が必要 | 今回は採らない（CSP 未決定） |

3 経路とも今回の受容対象外とし、どれを採るかは ADR-0253 の前提 1 の決定時に判断する。
比較表は選択の確定ではなく、判断材料の記録である。

### npm 経路の追加要件

- JS bundler の導入と `package-lock.json` の追跡。
- `trunk build` と bundler の順序、`--offline --locked` の再現性維持（ADR-0237）。
- web クレートに npm / Node が無い現在のビルド構成との整合。

### internals 経路の留意点

- 非公式のため API の存在・形状・安定性を公式ドキュメントで保証できない。
- 「常に注入される」という前提は今回の実機検証で確認していない。
- Tauri upgrade 時の契約（変更検知方法と移行手順）を未決定として残す。

## 未決定

- ~~invoke 経路の選択~~ は [ADR-0270](0270-tauri-global-ipc-path.md) で決定済み。
- Trunk inline script への CSP nonce / hash の適用方法。
- `index.html` の meta CSP と `tauri.conf.json` の config CSP の一本化。
- `withGlobalTauri` が注入する script の CSP 適合（nonce / hash の付与経路）。
- 経路と CSP の対応を検査する静的ゲートの追加有無。
- 実 Tauri WebView での IPC 往路検証（desktop で `Err("desktop only")` にならないこと）。
- `__TAURI_INTERNALS__` を採る場合の upgrade 契約（変更検知・移行手順）。

## 根拠

- ADR-0091: Web 版 CSP meta の追加。meta / config の二重管理が本案の未決定。
- ADR-0130: desktop CSP の `wasm-unsafe-eval` 対応。既存 `script-src` の出典。
- ADR-0191: 外部フォント削除と CSP のローカル完結化。CSP 変更の基本原則。
- ADR-0253: desktop 保存 UI の段階設計。実装前提 1 と実装前提 2〜6 の順序。
- Tauri 公式 "Calling Rust from the Frontend":
  <https://v2.tauri.app/develop/calling-rust/> — グローバル経路には
  `app.withGlobalTauri: true` が必要と明記。
- Tauri 公式 "Content Security Policy": <https://v2.tauri.app/security/csp/> —
  ローカル script は hash、外部 script / style は nonce で参照され、
  Wasm では `'wasm-unsafe-eval'` が必要と明記。

## 影響

- 本次は設計のみ。コード・CSP・capability・E2E は未変更で、E2E 255 件は不変。
- desktop の `fetch_config()` が往路未接続である可能性を開示した（実機未検証）。
- ADR-0253 の実装前提 1 が未決定のまま残り、保存 UI 接続は後続に留まる。
- browser 単体の fail-closed と読み取り専用契約は従来どおり維持する。

## 検証

- 文書形式: OKF frontmatter（`type: Decision`）、`##` からの見出し階段、RFC 3339 `+09:00`。
- `npx markdownlint-cli2 ...`: 0 issues。
- 根拠としたコード行は `web/src/lib.rs:4096-4121` と `desktop/tauri.conf.json` を直接確認。
- 実 Tauri WebView の往路検証と npm / internals の実測は未実施。
