---
type: Decision
title: "ADR-0200: send_wrapper の直接依存化"
description: "Leptos の doc(hidden) re-export を避け、send_wrapper を web クレートの直接依存として使う決定を記録する。"
tags:
  - ADR
  - deps
  - web
generated:
  by: dsh
  at: "2026-11-19T08:58:00+09:00"
status: accepted
---

## 背景

`web/src/lib.rs` は `leptos::__reexports::send_wrapper::SendWrapper` を使っていた
（ADR-0166 / 0192）。これは Leptos の `#[doc(hidden)]` re-export であり、
Leptos 更新時にパスが変わるとコンパイルエラーになる（R6 TD-M07）。
Leptos の re-export の実体は `send_wrapper 0.6.0` で、既に Cargo.lock に存在する。

## 決定

- `web/Cargo.toml` に `send_wrapper = "0.6"` を直接依存として追加する。
- `web/src/lib.rs` の import を `use send_wrapper::SendWrapper;` へ変更する。
- Cargo.lock の差分は `openrustbooks-web` の dependencies に 1 行追加するのみとし、
  新規クレートは増やさない。
- `take(self)` / `Deref` / `DerefMut` による既存 cleanup 挙動は変更しない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 直接依存 0.6（採用） | doc(hidden) パスから脱却、実体は同一クレートで挙動不変 |
| gloo-utils 経由 | gloo は SendWrapper を公開していない |
| 自作 SendWrapper | unsafe が必要で禁止事項に反する |
| `=0.6.0` 完全一致 | 将来 0.6.x が出た際に二重バージョン化するリスク |

## 影響

- Leptos が re-export パスを変更しても web クレートは影響を受けない。
- `send_wrapper` 0.x の API 変更時は cleanup コードの再確認が必要（未決定）。
- THIRD_PARTY_NOTICES には `send_wrapper 0.6.0` が既に記載済みで、更新不要。
- 新規 unsafe なし、E2E 251 維持。

## 検証（Runtime Proof）

- `grep -rn '__reexports::send_wrapper' web/src` → 0 件。
- Cargo.lock: package 数 793 → 793（不変）、
  package 名ハッシュ不変、追加 1 行を除く SHA-256 は更新前と完全一致。
- `cargo fmt --all -- --check`、
  `cargo check --all-targets --locked`、
  `cargo test --all-targets --locked`、
  `cargo clippy --all-targets --locked -- -D warnings`、
  wasm32 core / web check: すべて rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **251 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- Leptos 0.9 等への更新時に `send_wrapper` のバージョンを揃える基準。
- `send_wrapper` 0.x の破壊的変更時の cleanup 再確認手順。
- Cargo.lock 更新手順の Runbook 化（`--locked` 前に非 locked check が必要）。
- workspace.dependencies への昇格（現状は web のみ使用のため不要）。
