---
type: Decision
title: "ADR-0126: CI での脆弱性監査有効化"
description: "共通ワークフローの audit 入力を有効化し、CI でも cargo audit を実行する決定を記録する。"
tags:
  - ADR
  - ci
  - security
generated:
  by: dsh
  at: "2026-11-18T09:00:00+09:00"
status: accepted
---

## 背景

ADR-0125 でローカルの `cargo audit` ゲートを追加したが、CI は
billing issue で実行できず、ゲート化も未決定だった。
`.github/workflows/ci.yml` は共通ワークフロー
`Seduce0068/workflows/rust-gates.yml@v1` に委譲している。

## 調査結果（ci-workflow-researcher）

- `extra_gates` は `type: string` で、`run:` にそのまま展開される
  （L28–32, L157–162）。`cargo audit` と書くだけでは cargo-audit が
  インストールされず失敗する。
- 共通ワークフローは `audit: boolean`（既定 false）入力を持ち、
  `audit: true` のとき taiki-e/install-action で cargo-audit を導入し
  `cargo audit` を実行する（L53–57, L146–155）。
- 共通側の `self-test.yml` の `gates-audit` ジョブで audit パスが
  回帰テスト済み。
- `--deny warnings` は ADR-0125 のとおり使わない（既存警告 10 件が baseline）。

## 決定

- `.github/workflows/ci.yml` の `with:` ブロック先頭に `audit: true` を追加する。
- `extra_gates` の既存 2 行（wasm32 check）、`@v1` ピン留め、
  `permissions` / `concurrency` は変更しない。
- 共通ワークフロー自体は変更しない（グローバル指示 5）。
- ADR-0125 の「CI ワークフローは変更しない」は本 ADR で上書きし、
  未決定だった「CI ゲート化」を解消する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| `audit: true`（採用） | 共通側の既存入力で、インストールと実行が保証される |
| `extra_gates` に `cargo audit` | cargo-audit 未導入で失敗 |
| `--deny warnings` 併用 | 既存警告 10 件で即失敗する |

## 影響

- CI が脆弱性スキャンを含むようになる（脆弱性検出時はジョブ失敗）。
- advisory DB の git fetch が CI ランナーで発生するため、
  ネットワーク障害時は失敗し得る（fail-closed）。
- billing issue 中は実際の CI 実行結果を確認できない。

## 検証

- YAML を `yaml.safe_load` でパース成功、`audit = True`（boolean）を確認。
- 差分は 1 行のみ（追加 1・削除 0）。
- ローカルの `./scripts/verify.sh`（6 ゲート＋wasm-test＋audit＋E2E 228）は成功。
- CI は billing issue で未実行（ci-audit-auditor の指摘どおり、
  復帰後に `workflow_dispatch` で検証する責任を残す）。

## 未決定

- billing 解消後の CI 実行検証（責任者・手順・期限）。
- advisory DB 更新頻度とネットワーク障害時のリトライ方針。
- wasm-test の CI 導入（ADR-0119 から継続）。
- 警告 10 件（unmaintained 等）の対応（ADR-0086 / 0125 から継続）。
