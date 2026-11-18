---
type: Decision
title: "ADR-0196: wasm-pack test の CI ゲート化"
description: "wasm-bindgen-test を CI でも実行するため、共通スクリプトを verify.sh と extra_gates から呼ぶ決定を記録する。"
tags:
  - ADR
  - ci
  - wasm
generated:
  by: dsh
  at: "2026-11-19T08:08:00+09:00"
status: accepted
---

## 背景

`verify.sh` は `wasm-pack test --node --locked` を実行するが、
CI は `rust-gates.yml@v1` に委譲しており `extra_gates` には
wasm32 check 2 行しか無かった。
共通ワークフローは wasm-pack を導入しないため、
wasm-bindgen-test 10 件が CI で実行されていなかった（R6: TD-M05、
ADR-0119 の未決定）。

## 決定

- `scripts/ci-wasm-test.sh` を追加する:
  - `#!/usr/bin/env bash` + `set -euo pipefail`
  - スクリプト位置基準で `ROOT_DIR` を算出（CWD 非依存）。
  - `command -v wasm-pack` が無い場合のみ
    `cargo install wasm-pack --locked --version 0.13.1`。
  - `cd core && wasm-pack test --node --locked`。
- `scripts/verify.sh` の wasm-test ゲートは同スクリプトを呼ぶ薄いラッパーへ変更し、
  単一情報源とする。
- `.github/workflows/ci.yml` の `extra_gates` に同スクリプトを追加する。
  - 実行順は verify.sh と同じく
    rumdl → wasm-core check → wasm-web check → wasm test（末尾）。
  - 共通ワークフロー自体（外部リポジトリ）は変更しない。
- CI YAML の冒頭コメントを実際のゲート構成へ更新する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 独立スクリプト＋ verify / CI から利用（採用） | 単一情報源でローカルと CI の乖離を防ぐ |
| extra_gates に直接コマンドを書く | verify.sh と重複しドリフトする |
| 共通ワークフローを変更 | 外部リポジトリ／v2 が必要で範囲外 |

## 影響

- CI クリーンランナーでは wasm-pack 0.13.1 のソースビルドが毎回走る
  （時間・ネットワーク依存、未計測）。
- 既存 wasm-pack が別バージョンでも再導入しないため version skew を許容する。
- wasm32 target は `wasm_targets: wasm32-unknown-unknown` で導入済み。
- Node は GitHub ランナーに同梱される前提（**推測**、実 CI 未検証）。
- 新規 Cargo 依存なし、unsafe なし。

## 検証（Runtime Proof）

- bash -n / shellcheck / shfmt -d: 両スクリプトすべて成功。
- PyYAML で ci.yml parse、`extra_gates` 4 行が期待順と完全一致。
- `./scripts/verify.sh` rc=0、
  wasm-test（wasm-bindgen-test 10 件）成功、E2E **250 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- CI billing 解消後の実 CI 検証。
- 共通ワークフロー変更（CI-H01 / `v1` pin）の要否。
- wasm-pack install の時間とキャッシュ導入。
- 0.13.1 から最新版への更新（ローカル整合を優先した pin）。
- chromedriver / headless Chrome 経路の要否（現在は Node のみ）。
- `lint.rs` の `#[tokio::test]` が wasm 側で実行されるかの確認。
