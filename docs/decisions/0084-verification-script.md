---
type: Decision
title: "ADR-0084: 一括検証スクリプト"
description: "全ゲート・Wasm ビルド・E2E を再現可能に実行するスクリプトを追加する決定を記録する。"
tags:
  - ADR
  - tooling
  - testing
generated:
  by: dsh
  at: "2026-11-17T21:25:00+09:00"
status: accepted
---

## 背景

別セッションの調査レポートで「E2E・`trunk build`・`cargo audit` が CI ゲート外」
と指摘された。リポジトリ内には検証スクリプトが無く、`/tmp/openrustbooks-gates.sh`
が一時的に使われているだけだった。

## 事前調査

`find . -maxdepth 3 -type f \( -name '*.sh' -o -name 'Makefile' -o -name 'justfile' \)`
の結果、リポジトリ内に既存の検証スクリプトは無い。
既存の Web/E2E ハーネス（`tools/verify-web-reader.py`）は再利用し、
新規依存は追加しない。

## 決定

- `scripts/verify.sh` を追加し、次を順に実行する。
  1. 6 ゲート（fmt / check / test / clippy / wasm32 core / wasm32 web）
  2. `trunk build --offline`（一時ディレクトリへ出力）
  3. `python3 tools/verify-web-reader.py --dist <出力先>`
- ゲート失敗時は E2E を実行せず終了する。
- 一時ディレクトリは trap で必ず削除する。
- wasm-bindgen-cli が `$HOME/.cache/trunk/wasm-bindgen-0.2.129` にある場合は
  PATH に追加する（環境固有の対応）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| リポジトリ内にスクリプトを追加（採用） | 誰でも同じ検証を再現できる |
| `/tmp` の一時スクリプトに頼る | 再現性が無く、環境に依存する |
| CI ワークフローを変更 | 共通ワークフロー（ADR の方針）を変更する必要があり大規模 |

## 影響

- `./scripts/verify.sh` で全検証を一括実行できる。
- CI の構成自体は変わらない（ローカル検証の再現性が向上）。
- 追加依存は無い。

## 検証

- `bash -n` / `shellcheck` / `shfmt -d` が通る。
- `./scripts/verify.sh` を実行し、6 ゲート・trunk build・E2E 219 項目が
  すべて成功した。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `cargo audit` のゲート追加（新規ツール導入の検討が必要）。
- CI ゲートへの E2E / trunk build 組み込み（共通ワークフロー側の変更）。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
