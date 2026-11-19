---
type: Decision
title: "ADR-0258: feature が一致しない依存の workspace 化を見送り現状を設計受容とする"
description: "chrono / uuid / wasm dev tokio の feature 差分（member の加算／限定）を記録し、workspace 化せず現状を設計受容とする決定を記録する。"
tags:
  - ADR
  - deps
  - cargo
  - wasm
generated:
  by: dsh
  at: "2026-11-20T02:03:29+09:00"
status: accepted
---

## 背景

ADR-0246 は feature 差分が無い 18 依存を `workspace.dependencies` 参照へ統一し、
feature が一致しない chrono / uuid / wasm dev tokio と rumdl を明示的に除外した
（R6 R2-crates#5）。
3 依存はいずれも「feature が一致しない」に分類されるが、差分の向きは異なる。
chrono / uuid は member 側が feature を加算し、wasm dev tokio は member 側が限定している。
本 ADR はこの現状を記録し、workspace 化を見送る判断を設計受容として残す。

## 現状

宣言の根拠は `Cargo.toml:21` / `:25` / `:26` と `core/Cargo.toml:49` / `:50` /
`:57` / `:67` / `:73` / `:92` である。

| 依存 | workspace 定義 | member 側の宣言 | 差分の向き |
| --- | --- | --- | --- |
| chrono | `["serde"]` | core / web: `["serde", "wasmbind"]` | member が加算 |
| uuid | `["serde", "v4"]` | core / web: `["serde", "v4", "js"]` | member が加算 |
| tokio（wasm dev） | `["full"]` | core wasm dev: `["macros", "rt"]` | member が限定（workspace 化で full が加算） |
| rumdl | 定義なし | core native / wasm: `default-features = false` | workspace に無く ADR-0242 / `check-rumdl-featureless.sh` で除外 |

- 3 依存のうち rumdl を除く上記が ADR-0246 の対象外となった feature 不一致依存である。
- chrono / uuid は `web/Cargo.toml:60` / `:61` でも同じ feature を直接宣言しており、
  workspace 定義との差分は core と web の 2 箇所にある。
- desktop の chrono は `desktop/Cargo.toml:24` で workspace 参照を使い、feature は `serde` のみ。
- native の tokio は既に workspace 参照（`core/Cargo.toml:57`）であり、対象は wasm dev のみ。
- rumdl は featureless 2 行をゲートが厳密検査するため、workspace 版を追加すると
  decl_count=3 で失敗する（ADR-0195 / 0242）。

## 決定

- chrono / uuid / wasm dev tokio は member 側の直接宣言を維持し、workspace 化しない。
- 現状の feature 差分を設計受容として記録し、R2-crates#5 を設計受容・残置で完了とする。
- rumdl は workspace 定義が無いまま featureless 2 行を維持し、ADR-0242 / 0246 の除外を継承する。
- `Cargo.toml` / `Cargo.lock` / コード / E2E は本次で変更しない。

## 根拠

- Cargo の feature は和集合（加算）で解決され、member 側から有効済み feature を削除できない。
- chrono / uuid（member が加算）: workspace 定義へ member 側の feature を上げて一本化すると、
  desktop の chrono（`desktop/Cargo.toml:24`）へ wasmbind、workspace uuid を参照する
  member へ js が波及する。和集合を逆方向へ削除することはできない。
- wasm dev tokio（member が限定）: `{ workspace = true }` へ変更すると、member の
  `["macros", "rt"]` の上へ workspace の full が加算される。
- 加算による具体的なビルド影響（依存の増減・挙動差・バンドルサイズ）は未検証の推測であり、
  断定しない。根拠に用いるのは Cargo の feature 加算性のみで、
  ADR-0242 の「推測・未検証」を継続する。
- ADR-0246 の第一段統一は完全一致 18 件に限定しており、本決定はその除外判断を継承する。
- ADR-0242 は feature 加算性を理由に rumdl の workspace 化を否定しており、方針は同一である。
- MSRV 1.95（ADR-0214）における workspace 側 `default-features` の override 制限も
  同じく **未検証の推測** であり、本決定の根拠には用いない（ADR-0242 / 0246 を継続）。

## 選択肢

| 選択肢 | 判断 |
| --- | --- |
| 全 workspace 化 | 却下。chrono / uuid は member の加算分が波及し、wasm dev tokio は full が加算される。 |
| 現状維持（採用） | member 直接宣言を残し、意味とゲートを変えずに差分を設計受容として記録する。 |
| workspace 定義分割 | 却下。用途別に別名の定義を増やしても加算性は解消せず、定義が分散して ADR-0246 の意図に反する。 |

## 影響

- `Cargo.toml` / `Cargo.lock` / `core/Cargo.toml` / `web/Cargo.toml` は差分なし。
- `check-rumdl-featureless.sh` の期待 2 行を維持。
- E2E 255 件の期待値は据え置き（文書のみの変更）。
- ADR-0246 の未決定だった 3 依存を設計受容へ移し、再評価条件を下記の未決定へ整理する。

## 検証（Runtime Proof）

- `bash scripts/check-rumdl-featureless.sh`: 期待 2 行を維持し rc=0。
- `cargo metadata --locked --no-deps --format-version 1`: rc=0（manifest の整合性）。
- `npx markdownlint-cli2 'docs/**/*.md' README.md todo.md`: 0 issues。

## 未決定

- MSRV 1.95 の workspace 側 `default-features` override 動作の実測（未検証、
  ADR-0242 / 0246 を継続）。
- Cargo の workspace feature マージ／override 仕様が変更された場合の再評価。
- chrono / uuid / tokio の依存更新や wasmbind / js の必要性変化による再評価。
- 加算による具体的なビルド影響（依存・挙動・バンドルサイズ）の実測
  （ADR-0160 / 0242 / 0256 を継続）。

## 参照

- [ADR-0214: MSRV 1.95 の宣言](0214-msrv.md)
- [ADR-0242: rumdl feature 選択の現状と ADR-0003 の履歴化](0242-rumdl-feature-status.md)
- [ADR-0246: workspace 依存の第一段統一](0246-workspace-dependency-unification.md)
