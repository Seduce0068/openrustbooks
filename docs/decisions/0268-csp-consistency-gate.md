---
type: Decision
title: "ADR-0268: CSP 整合性の静的ゲート"
description: "meta CSP と desktop config CSP の一致・許可オリジンを fail-closed で検査する静的ゲートを記録する。"
tags:
  - ADR
  - csp
  - ci
generated:
  by: dsh
  at: "2026-11-20T04:32:00+09:00"
status: accepted
---

## 背景

ADR-0130 は desktop 側 CSP を直接検証する静的ゲートを未決定として残していた。
`web/index.html` の meta CSP と `desktop/tauri.conf.json` の config CSP は
意図的に allowlist が異なる（desktop は IPC / OpenAI / Ollama を許可）ため、
単純な全文一致ではなく、共通 directive と許可オリジンを検査する必要がある。

## 決定

- `scripts/check-csp-consistency.sh` を追加する。
- 検査項目:
  - meta CSP が 1 件だけ存在し、`content` が非空であること。
  - `tauri.conf.json` の `app.security.csp` が存在し、JSON が正常であること。
  - `script-src` / `style-src` が meta と config で一致すること（空白・順序は正規化）。
  - config に `default-src` / `img-src` / `font-src` / `connect-src` が存在すること。
  - meta に `http://` / `https://` が無いこと（ADR-0191）。
  - config のオリジンが許可リスト
    （`ipc:` / `http://ipc.localhost` / `https://api.openai.com` /
    `wss://api.openai.com` / `http://localhost:11434`）のみであること。
  - 既知 directive のみ使用し、`script-src` / `style-src` に
    `'unsafe-eval'` が無いこと（`wasm-unsafe-eval` は許可）。
- `verify.sh` と `ci.yml` の `extra_gates` から同じ script を呼ぶ。
- 共通ワークフローは変更しない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 静的ゲートを追加（採用） | desktop CSP の直接検証という ADR-0130 の未決定を解消できる |
| 両 CSP の全文一致 | IPC / AI endpoint の意図的差分を誤拒否する |
| E2E のみ | E2E は meta CSP を読み、desktop config を検証しない |

## 影響

- CSP 値、HTML、tauri.conf、E2E check 名・件数は変更しない（255 維持）。
- custom endpoint / 新規プロトコルを追加する場合は
  許可リスト更新が必須になる（未決定に記録）。
- HTML コメント内の `<meta>` も抽出対象のため、
  コメント化した CSP 例を書くと fail-closed で拒否される。

## 検証（Runtime Proof）

- `bash -n` / `shellcheck`: OK。
- 負向 test **15 ケース**（script-src 不一致、font-src 欠落、
  meta への https 追加、未知オリジン、不正 JSON、ファイル欠落、
  meta CSP 0/複数件、app.security.csp 欠落、未知 directive（meta/config）、
  両方への `'unsafe-eval'` 追加、正常/順序入替の陽性対照）: すべて期待 rc。
- `./scripts/verify.sh`: `csp-consistency rc=0`、全ゲート成功、
  E2E **255 passed / 0 failed**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- tauri.conf.json の plugins 設定は ADR-0280 の別ゲートで検査する。

- WebView 系統ごとの `wasm-unsafe-eval` 対応と
  meta/config 共存時の実効（ADR-0130 を継続）。
- nonce/hash 化（ADR-0091）と `withGlobalTauri` の同時実施（ADR-0264）。
- 許可リスト更新手続き（custom endpoint / ADR-0101 と連携）。
- `web/dist/index.html` の meta も検査対象に含めるか（ADR-0191）。
- HTML コメント内の `<meta>` 抽出の誤検知をどう扱うか。
