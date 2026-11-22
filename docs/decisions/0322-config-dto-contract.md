---
type: Decision
title: "ADR-0322: Config DTO 契約の静的検証"
description: "desktop / web の ConfigView / ConfigPatch DTO を 12 ペアで比較し、フィールド drift を CI で検出する決定。"
tags:
  - ADR
  - IPC
  - CI
generated:
  by: dsh
  at: "2026-11-22T14:28:00+09:00"
status: accepted
---

## 背景

ConfigView / ConfigPatch が desktop と web で手書き重複し、
フィールド削除・改名・型変更がコンパイル時に検出されなかった。
実行時の deserialize 失敗や未知フィールドの黙った値落ちが懸念された。

## 決定

- `scripts/check-config-dto-contract.sh` を追加する。
- 12 ペア（ConfigView / ConfigPatch / General / Ai / OpenAi / Ollama /
  Lint / Storage / Editor / Reader / LocalRoot / GoogleDrive）の
  フィールド名集合を完全一致で比較する。
- 片側例外は `revision`（web CAS 補助）と `expected`（desktop CAS 入力）のみ。
- struct 消失 / 重複定義 / 未知・欠落フィールド / 解釈不能行を fail-closed で拒否する。
- desktop に ConfigView JSON キー集合 test、web に SettingsView 全フィールド
  fixture の往復 test とネスト欠落拒否 test を追加する。
- verify.sh と CI extra_gates の両方に同じゲートを配線する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 静的フィールド比較 + 境界 test（採用） | 新依存なしで drift を検出できる |
| 共通 DTO crate へ昇格 | core の wasm/native 境界を広げるため見送り |
| 実行時のみで検出 | CI で防げない |

## 検証（Runtime Proof）

- 負向 7 ケース（追加 / 削除 / リネーム / struct 消失 / 例外逸脱 /
  reader 削除 / ConfigPatch.expected 改名）すべて検出。
- desktop test: 165 passed / 0 failed。
- web test: 19 passed / 0 failed。
- wasm core / web check: rc=0。
- `npx tsc --noEmit`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、
  config-dto-contract rc=0、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- 型トークン自体の比較（現状はフィールド名のみ）。
- SettingsView への `deny_unknown_fields` 追加（静的ゲートで代用中）。
