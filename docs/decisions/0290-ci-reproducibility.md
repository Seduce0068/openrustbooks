---
type: Decision
title: "ADR-0290: CI / WDIO の再現性と安全性の改善"
description: "wasm-pack 固定、lock 不変検証、dependabot npm、WDIO cleanup の安全化で CI の再現性を高める決定。"
tags:
  - ADR
  - CI
  - WDIO
generated:
  by: dsh
  at: "2026-11-22T04:07:00+09:00"
status: accepted
---

## 背景

ci-wasm-test.sh は既存 wasm-pack のバージョンを検査せず、
ci-wdio-test.sh は `--locked` を付けず、port 1420 の listener を無差別に kill していた。
dependabot は cargo / actions のみで、WDIO の npm 依存が自動更新されていなかった。

## 決定

- wasm-pack は 0.13.1 のみを許容し、不一致 / 未導入時の導入後も再検証する。
- WDIO の desktop build は `--locked` を使い、
  Cargo.lock / package-lock.json の HEAD 差分をビルド前後で検査する。
- port 1420 はビルド前と trunk 起動直前に検査し、
  readiness は自 PGID の listener であることを要求する。
- trunk は独立 process group で起動し、cleanup は自グループのみ TERM / KILL する。
- trunk ログは一時ファイルへ出力し、失敗時のみ表示して常に削除する。
- dependabot に npm / root / weekly / `wdio` group を追加する。
- runbook は `npm ci` と `npm ls --depth=0` を前提にする。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 各ゲートで fail-closed に固定（採用） | ローカル / CI の環境差を減らせる |
| 共通 workflow を直接変更 | 共通指示 5 に反するため却下 |
| バージョンを案内のみ | 再現性が担保できないため却下 |

## 影響

- `ss` / `ps` / `setsid` が WDIO ゲートの前提になる。
- `npm ci` 済みでない場合、`npm ls --depth=0` が失敗する。
- CARGO_HOME が既定でない環境でも wasm-pack を検出する。
- 共通 workflow 自体は変更しない。

## 検証（Runtime Proof）

- `bash -n` / `shellcheck`: ci-wasm-test / ci-wdio-test / verify.sh すべて rc=0。
- `shfmt -d`: 変更 2 script は差分なし。
- PyYAML parse: cargo / actions / npm を確認。
- markdownlint: runbook rc=0。
- バージョン判定サニティ: 0.13.1 と 0.13.1 (build info) は許可、
  0.13.10 / 0.14.0 は拒否。
- `npm ls --depth=0`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- 共通 workflow 側での npm ci / Node setup の保証。
- GitHub Actions  billing 解消後の実 CI 検証。
- E2E 一時 DIST と web/dist の二重 trunk build の統合。
