---
type: Decision
title: "ADR-0088: 配布 WASM の開発パス除去"
description: "配布 WASM に含まれる開発ホスト絶対パスを除去する決定を記録する。"
tags:
  - ADR
  - security
  - wasm
  - tooling
generated:
  by: dsh
  at: "2026-11-17T22:40:00+09:00"
status: accepted
---

## 背景

配布用 WASM（`web/dist/*_bg.wasm`）に `/home/user/...` などの開発ホスト
絶対パスが 1811 件含まれていた（todo.md E01）。
デバッグ情報やパニックメッセージから開発環境のパスが漏れる可能性がある。

## 決定

- `scripts/verify.sh` の trunk build 時に
  `RUSTFLAGS="--remap-path-prefix=$HOME=~"` を指定する。
- ビルド後に WASM と JS グルーを走査し、開発ホーム絶対パスが含まれていれば失敗とする（ADR-0152 で fail-closed 化）。
- `strip = true`（release）と `debug = "line-tables-only"（dev）` の既存設定は維持する。
- 開発パスをリポジトリにハードコードしないため、Cargo 設定ではなく
  スクリプトで `$HOME` を使う。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| `RUSTFLAGS` で動的 remap（採用） | 環境に依存せず、配布ビルドで確実に除去できる |
| `-C remap-path-prefix=...` | rustc が「unknown codegen option」で失敗する（誤り） |
| `.cargo/config.toml` に絶対パスを記載 | 開発者ごとのパスが残り、ポータビリティが失われる |
| release ビルドに切り替える | 現在の環境では `wasm-opt` が offline で取得できず失敗する |

## 影響

- `./scripts/verify.sh` が生成する WASM には開発ホスト絶対パスが含まれない。
- 手動で `trunk build` する場合は、同じ `RUSTFLAGS` を付ける必要がある。
- 既存の 126 テスト、E2E 223 項目はそのまま通る。

## 検証

- `RUSTFLAGS="--remap-path-prefix=$HOME=~" trunk build --offline` で
  生成した WASM の `/home/user` 出現回数が 0 件であることを確認した。
- `./scripts/verify.sh` を実行し、6 ゲート・trunk build・WASM パス検査・
  E2E 223 項目がすべて成功した。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 手動ビルド用の `scripts/build-web.sh` を追加するか。
- release ビルドでの `wasm-opt` 導入（オフライン環境での取得方法）。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
