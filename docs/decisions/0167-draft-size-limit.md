---
type: Decision
title: "ADR-0167: エディタ下書きの容量上限"
description: "下書き保存に 1 MiB の UTF-8 バイト上限を設け、超過時は保存せず固定文言で通知する決定を記録する。"
tags:
  - ADR
  - web
  - data
generated:
  by: dsh
  at: "2026-11-18T23:55:00+09:00"
status: accepted
---

## 背景

ADR-0164 で localStorage ドラフト保存を実装したが、本文の容量上限が無く、
巨大な入力が QuotaExceededError を起こし得た（ADR-0154 の通知は出るが予防策が無い）。

## 決定

- `EDITOR_DRAFT_MAX_BYTES = 1_048_576`（1 MiB）を追加。
- `save_editor_draft()` の先頭で `content.len() > MAX` を判定する。
  - `len()` は UTF-8 バイト数（文字数や UTF-16 code unit ではない）。
  - `== 1 MiB` は保存許可、`> 1 MiB` のみ拒否。
- 超過時は `set_item` を呼ばずに
  `report_save_failure("下書き", SaveErrorTarget::Editor)` を発行し false を返す。
- Editor の Save ハンドラでは:
  - 上限超過: 固定文言「下書きが大きすぎて保存できません」。
  - それ以外の失敗: 既存の `SAVE_ERROR_MESSAGE`
    （「保存に失敗しました。ブラウザの設定または空き容量を確認してください。」）を維持。
- 本文・サイズ値は status / console / イベントへ載せない
  （ADR-0154 / 0165 の redaction 原則を維持）。
- E2E に 1 件追加し、期待総数 241 → **242**（check 225 + equal 17）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| UTF-8 バイトで 1 MiB（採用） | quota に最も近く、境界が明確 |
| 文字数 | 日本語で実バイトが約 3 倍になり quota を圧迫 |
| EPUB と同じ 256 MiB | 下書きには過大 |

## 影響

- 上限を超える下書きは保存されず、既存の保存済み値は更新されない
  （E2E で番兵による確認）。
- 既存の下書き保存／復元（短文）は不変。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- E2E「1 MiB 超の下書きは保存されず固定文言の status が出る」が ok。
- E2E **242 項目すべて成功**。
- 全単体テスト core 202 + desktop 56 + web 2、wasm-test 10、
  audit 脆弱性 0 件。
- `python3 -m py_compile`、`cargo fmt --all -- --check`、wasm check、
  `./scripts/verify.sh`: すべて成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 上限値を定数ではなくユーザー設定にするか。
- 自動保存／デバウンス時の超過扱い（ADR-0164 から継続）。
- ~~既に 1 MiB 超の保存済み下書きの復元~~ ADR-0170 で本文保持＋固定警告を実装（prune は未決定）。
- 位置／設定／履歴キーへの容量上限適用、localStorage 全体 quota の合計管理。
- サイズ値を出さない制約と利用者案内の両立。
