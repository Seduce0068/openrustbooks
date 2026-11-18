---
type: Decision
title: "ADR-0170: 上限超過下書きの復元時警告"
description: "1 MiB 超の保存済み下書きは本文を保持したまま復元し、固定警告のみを表示する決定を記録する。"
tags:
  - ADR
  - web
  - data
generated:
  by: dsh
  at: "2026-11-19T00:45:00+09:00"
status: accepted
---

## 背景

ADR-0167 で新規保存は 1 MiB に制限したが、
`load_editor_draft()` は保存済みの 1 MiB 超本文も無警告で復元していた。
復元側で削除／拒否すると利用者のデータを失うリスクがある。

## 決定

- 保存済み下書きが `EDITOR_DRAFT_MAX_BYTES` を超えていても、
  **本文はそのまま復元**する（`set_content` + `textarea.set_value`）。
- 復元時に上限超えを検出したら、`.editor-status` に固定文言
  「保存済みの下書きが大きすぎます。編集して保存してください。」を表示する。
- 過剰な下書きの自動削除・切断・prune・再保存は行わない。
- 本文・サイズ値・キー名は status / console / イベントへ載せない
  （ADR-0154 / 0165 / 0167 の redaction 原則を維持）。
- E2E に 1 件追加し、期待総数 243 → **244**（check 227 + equal 17）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 本文復元＋固定警告（採用） | データ損失なく注意を促せる |
| 復元拒否／削除／切断 | 利用者のデータを壊す |
| サイズ値を表示 | redaction 方針に反する |

## 影響

- ADR-0167 の「prune」未決定は残る（自動削除しない方針を明示）。
- E2E では復元後も localStorage の自キーが 1 MiB 超のまま残ることを
  長さのみで検証（本文は CDP 応答に出さない）。
- `finally` で自キーと textarea をリセットし、後続テストへの順序依存を低減。

## 検証（Runtime Proof）

- E2E「1 MiB 超の保存済み下書きが本文ごと復元され固定文言の警告が出る」が ok。
- E2E **244 項目すべて成功**。
- 全単体テスト core 202 + desktop 56 + web 2、wasm-test 10、
  audit 脆弱性 0 件。
- `python3 -m py_compile`、`cargo fmt --all -- --check`、wasm check、
  `./scripts/verify.sh`: すべて成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 過剰な下書きの prune／自動削除方針（今回は明示的に行わない）。
- 上限のユーザー設定化、自動保存／デバウンス時の超過扱い
  （ADR-0164 / 0167 から継続）。
- 他キーへの容量上限適用、localStorage 全体 quota の合計管理。
- `doc_id` 変更時に古い警告をクリアするか（SPA パラメータ変更の可能性、
  **推測**）。
