---
type: Decision
title: "ADR-0239: Cargo をバージョンの単一情報源とする"
description: "tauri.conf.json の version を削除し、Cargo workspace version へのフォールバックと静的ゲートで drift を防ぐ決定を記録する。"
tags:
  - ADR
  - release
  - ci
generated:
  by: dsh
  at: "2026-11-19T19:53:00+09:00"
status: accepted
---

## 背景

workspace `Cargo.toml` と `desktop/tauri.conf.json` が
ともに `0.1.0` を保持する二重管理だった（R6 R3-release#3）。
Tauri 2 は `version` キー省略時に Cargo.toml のバージョンへ
フォールバックする公式動作を持つ一方、
JSON 内での環境変数展開はサポートされない。
`generate_context!()` は文字列リテラルを要求し、
OUT_DIR 生成 config をマクロから参照できないため、
build.rs 生成案は却下した。

## 決定

- `desktop/tauri.conf.json` から `"version": "0.1.0"` を削除する。
- Cargo workspace version を単一情報源とする
  （`desktop/Cargo.toml` / `core/Cargo.toml` は `version = { workspace = true }`）。
- `scripts/check-version-consistency.sh` を追加し、
  tauri.conf.json のトップレベルに `version` が無いことを fail-closed で検査する。
  - 第 1 引数で対象 JSON を差し替え可能（負向 test 用）。
  - ファイル欠落 / JSON parse 失敗 / cargo metadata 失敗も失敗扱い。
  - `jq` 非依存、Python 3 stdlib を使用。
- `scripts/verify.sh` と `.github/workflows/ci.yml` の `extra_gates` から
  同じスクリプトを呼ぶ。共通ワークフロー `rust-gates.yml@v1` は変更しない。
- Tauri config への build.rs 生成・`TAURI_CONFIG` 上書きは採用しない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| version 削除＋静的ゲート（採用） | 公式 fallback を利用し drift を自動検出できる |
| JSON に `$CARGO_PKG_VERSION` を埋め込む | Tauri config は展開をサポートしない |
| build.rs で tauri.conf を生成 | `generate_context!` の literal path 制約と設定ソース二重化で却下 |
| 現状維持 | workspace 更新忘れを検出できない |

## 影響

- Cargo workspace version を上げれば Tauri 側も同じ値になる。
- `core::utils::version()` も `env!("CARGO_PKG_VERSION")` で一致する。
- `tauri build` / bundler における version 省略時の挙動は
  実機で未検証（tauri-cli 未インストール）。
- 既存 static gate が 1 つ増え、E2E 254 は維持される。

## 検証（Runtime Proof）

- 負向 test: `version` 混入（0.1.0 / 9.9.9）、不正 JSON、
  トップレベル配列、ファイル欠落はいずれも rc=1。
- 通常 run: rc=0、Cargo version `0.1.0` をログ出力。
- `bash -n` / shellcheck: 0 issues。
- `./scripts/verify.sh`: 全ゲート成功、E2E **254 項目すべて成功**。
- Cargo.lock: 差分なし。

## 未決定

- Release ワークフロー（公開鍵・署名 secrets・`contents: write`・ホスティング・
  billing 責任者と期限。ADR-0222 / 0232 を継続）。
- minisign の鍵生成・ローテーション・kill switch と
  `require_signed_version` の有効化時期。
- update manifest（HTTPS endpoint、CSP `connect-src`、
  `check_updates` の配線または削除、`createUpdaterArtifacts` の実効化）。
- `tauri build` での version fallback 実証。
- platform-specific config（`tauri.linux.conf.json` 等）への走査拡張。
