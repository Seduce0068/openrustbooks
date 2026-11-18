---
type: Decision
title: "ADR-0185: cargo-audit ゲートの fail-closed 化"
description: "cargo-audit 未インストール時に検証をスキップせず失敗させる決定を記録する。"
tags:
  - ADR
  - ci
  - security
generated:
  by: dsh
  at: "2026-11-19T05:21:00+09:00"
status: accepted
---

## 背景

`scripts/verify.sh` は `command -v cargo-audit` が失敗すると
`WARNING: cargo-audit not installed; skipping audit gate` を出すだけで
`fail=1` にしていなかった。ツール未導入のローカルでは
脆弱性検査なしで「全検証成功」と表示され、CI（`audit: true`）と結果が食い違う
（R4 レビュー指摘）。

## 決定

- `cargo-audit` が無い場合は次を出力してゲートを失敗させる:
  - `ERROR: cargo-audit is required; install with: cargo install cargo-audit --locked`
  - `===== START: audit =====` / `===== END: audit rc=127 =====`
    （`run` ヘルパーと同じ出力形式、127 は command not found の慣例値）
  - `fail=1`
- `--no-fetch` / 既知 baseline 警告 10 件の許可 / `--deny warnings` 不使用は維持。
- advisory DB が無い場合は `cargo audit` が rc!=0 を返し、
  `run` ヘルパーが `fail=1` にする（既存の fail-closed を維持）。
- **`--no-fetch` は stale 判定を無効化するため、DB の鮮度は検出しない**。
  定期的な `cargo audit update-db` の運用責任に委ねる
  （ADR-0125 / ADR-0086 の方針を継続）。
- README の前提条件に `cargo install cargo-audit --locked` を明記
  （既存記載を維持・強調）。
- CI 共通ワークフローは変更しない（`audit: true` 済み）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 未インストール時に fail（採用） | 「全検証成功」が脆弱性未検査を意味しないようにする |
| 警告してスキップ（旧挙動） | ローカルと CI の非対称、false green の原因 |
| `\|\| true` で握り潰す | fail-open になるため却下 |

## 影響

- `cargo-audit` 未導入の環境では `./scripts/verify.sh` が rc=1 になり、
  E2E へ進まない。
- ADR-0125:36-37 / 51 の旧記述（警告してスキップ）は
  本 ADR で上書きされる（旧 ADR は履歴として残す）。
- 新規依存なし。

## 検証（Runtime Proof）

- `bash -n scripts/verify.sh`、shellcheck / shfmt: 成功。
- PATH から `cargo-audit` のみ除外した一時シム環境で
  `HIDDEN_RC=1` を確認。
  ログに ERROR / `END: audit rc=127` /
  `ゲート失敗: E2E は実行しません` を確認。
- 通常環境: `./scripts/verify.sh` rc=0、
  audit は 1533 advisories / 793 crates をスキャンし脆弱性 0
  （baseline 警告 10）。E2E **249 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `--stale` を導入して古い advisory DB もゲート失敗にするか
  （現状は `--no-fetch` と併用不可、ADR-0125 の未決定を継続）。
- `rc=127` を専用終了コードへ変更するか。
- ADR-0086 / 0125 / 0126 の旧記述を直接更新するか（本 ADR で上書き済み）。
