---
type: Decision
title: "ADR-0309: 依存ゲートと既知例外の期限管理"
description: "serde_yaml / tikv-jemalloc-sys の依存経路を cargo tree で実測し、ID・理由・期限付き例外として管理する決定。"
tags:
  - ADR
  - CI
  - dependencies
generated:
  by: dsh
  at: "2026-11-22T09:55:00+09:00"
status: accepted
---

## 背景

`check-rumdl-featureless.sh` は Cargo.toml の featureless 宣言のみを検査し、
Cargo.lock に残る serde_yaml / tikv-jemalloc-sys を扱わなかった。
cargo audit の unmaintained 警告にも例外根拠・期限が無かった。

## 決定

- `scripts/check-dependency-hygiene.sh` を追加する。
- `cargo tree -i <crate> --prefix none --depth 1 --locked --offline` で
  逆依存経路を実測し、想定外の経路を fail-closed で拒否する。
- 既知例外は ID / 有効期限 / 許容経路 / 理由をテーブルで管理する。
  - EXC-DEP-001: serde_yaml ← rumdl v0.3.2。
  - EXC-DEP-002: tikv-jemalloc-sys ← tikv-jemallocator ← rumdl v0.3.2。
  - 期限: 2027-03-31（再レビュー期限）。
  - 理由: rumdl の非 optional ハード依存のため、
    `default-features = false` でも lock から消えない。
- 期限切れ / 未定義例外 / 想定外経路 / lock のみ残存 / テスト env の通常使用は
  すべて失敗する。
- テスト用 env は `DEP_HYGIENE_TEST_MODE=1` のときのみ有効。
- verify.sh と `.github/workflows/ci.yml` の `extra_gates` の両方に配線する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 実測 + 期限付き例外（採用） | 根拠と更新時点を残せる |
| lock grep のみ | 依存経路の変化を検出できない |
| 例外なしで全失敗 | 現行 rumdl 依存で verify が通らない |

## 検証（Runtime Proof）

- positive: 実 baseline rc=0 / fixture-mode rc=0。
- 負向: 期限切れ rc=1 / 想定外依存 rc=1 / 設定改変 rc=1 /
  テスト env 単独設定 rc=1 / fixture 欠落 rc=1。
- `bash -n` / `shellcheck`: rc=0。
- `shfmt -d`: 新規スクリプトに差分なし。
- `./scripts/verify.sh`: 全ゲート成功、
  dependency-hygiene rc=0、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- rumdl 側で serde_yaml / tikv-jemallocator が optional 化された場合の例外撤廃手順。
- cargo audit の unmaintained 警告に対する個別 ID / 期限管理。
