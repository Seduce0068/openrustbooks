---
type: Decision
title: "ADR-0326: wasm-bindgen CLI / runner の版固定検証"
description: "Cargo.lock を単一情報源として wasm-bindgen CLI と test runner の版を照合し、不一致を fail-closed で拒否する決定。"
tags:
  - ADR
  - CI
  - reproducibility
generated:
  by: dsh
  at: "2026-11-22T23:18:00+09:00"
status: accepted
---

## 背景

verify.sh / build-desktop.sh は `$HOME/.cache/trunk/wasm-bindgen-0.2.129` を
固定パスで PATH へ追加するだけだった。
Cargo.lock が更新されても古い CLI が黙って使われ、
wasm-pack の test runner も固定版のままだった。

## 決定

- Cargo.lock の `[[package]] name = "wasm-bindgen"` から version を抽出する。
- verify.sh / build-desktop.sh は抽出版からキャッシュ PATH を導出し、
  `wasm-bindgen --version` が Cargo.lock 版と一致するか検査する。
- ci-wasm-test.sh は同じ抽出ロジックで
  `wasm-bindgen-test-runner --version` を照合して runner を選択する。
- 抽出 0 件 / 複数件 / 不正 semver / CLI 不在 / version 実行失敗 /
  不一致はすべて fail-closed。
- 3 スクリプトの抽出関数は同一内容（SHA256 一致を実測で確認）。
- runbook から固定パス export 例を削除し、Cargo.lock 導出方式を記載する。
- `verify.sh --check-wasm-bindgen [<lockfile>]` で単独検査可能。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| Cargo.lock 抽出 + CLI 照合（採用） | 単一情報源で drift を検出できる |
| 固定パス継続 | lock 更新時に古い CLI が使われる |
| 共通スクリプトへ分離 | 書き込み範囲と既存構成を変えるため今回は見送り |

## 検証（Runtime Proof）

- 負向 16/16 + ci-wasm 11/11（CLI 不在 / 不一致 / lock 書換え /
  抽出失敗 / 0.2.122 / 0.2.1290 サフィックス）すべて期待 rc。
- `bash -n` / `shellcheck`: rc=0。
- `shfmt -d`: 新規 / 変更行に差分なし（verify.sh の既存 2 hunk のみ）。
- markdownlint: 0 issues。
- `./scripts/verify.sh`: 全ゲート成功、wasm-test rc=0、
  E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- 3 スクリプトの抽出関数を共通ファイルへ分離するか（現状はコメントで同期を運用）。
