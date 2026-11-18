---
type: Decision
title: "ADR-0180: エディタ下書きのバックアップ対応"
description: "バックアップ JSON に下書きを optional field として含め、復元時に現在値を保護する決定を記録する。"
tags:
  - ADR
  - web
  - data
generated:
  by: dsh
  at: "2026-11-19T04:10:00+09:00"
status: accepted
---

## 背景

Web エディタの唯一の保存先は localStorage の
`openrustbooks:editor:{doc_id}` だったが、`export_positions()` は
positions / settings / history のみを出力していた。
バックアップ復元時に本文が戻らず、サイトデータ削除・端末変更時に
下書きだけが復旧不能になる（R6: TD-H06）。

## 決定

- `EXPORT_VERSION` は **1 のまま**とし、`drafts` を optional field として追加する。
  - 旧バックアップ（`drafts` なし）は従来どおり読める。
  - 新エクスポートは `drafts: {}` も出すが、旧アプリは無視するため双方向互換。
- export は `openrustbooks:editor:` で始まるキーを
  `{ "drafts": { key: content } }` として含める。
  - 値が `EDITOR_DRAFT_MAX_BYTES`（1 MiB）を超えるものは除外（ADR-0167 / 0170）。
  - `count = positions.len() + drafts.len()` とし、positions が 0 件でも
    drafts があればエクスポートする。
- import は `drafts` が存在する場合:
  - キーが `openrustbooks:editor:` で始まること。
  - 値が文字列で 1 MiB 以下であること。
  - 現在 localStorage に同じキーが無いこと（ADR-0157 の現在値保護）。
  - 上記を満たす場合のみ `set_item()` する。
- `set_item()` 失敗は件数に含めず `*save_failed` を立て、
  通知は操作単位で 1 回だけ発行する（ADR-0169）。
- 戻り値 `(positions, history)` と呼び出し側 status 文言は維持する。
- E2E に 1 件追加し、期待総数 248 → **249**（check 232 + equal 17）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| version 1 のまま optional field（採用） | 旧バックアップ互換を壊さない |
| version 2 へ上げる | 既存 E2E / 利用者のバックアップを広く拒否する |
| 下書きを別ファイルにする | 利用者が二つのファイルを管理する必要がある |

## 影響

- バックアップ JSON には下書き本文が**平文**で含まれる。
  ただし status / console / エラー文言には本文・キー名・サイズ値を出さない。
- 下書きだけのバックアップを復元すると
  `Imported 0 reading positions and 0 histories.` と表示される
  （件数返却・status 文言を維持したため、未決定）。
- 現在値保護は下書きにも適用される（既存キーは上書きしない）。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- E2E「下書きがエクスポートされバックアップキー削除後にインポートで復元される」が ok。
  - export JSON に `drafts` が存在。
  - 1 MiB 超の下書きが export で除外される。
  - 改ざん JSON（`version:1`, `positions:{}`, drafts に 1 MiB 超＋正当値）で
    import 側ガードが働く。
  - 後始末は自キー `removeItem()` のみ（`localStorage.clear()` 不使用）。
- E2E **249 項目すべて成功**。
- 全単体テスト core 212 + desktop 59 + web 2、wasm-test 10、
  audit 脆弱性 0 件。
- `python3 -m py_compile`、`cargo fmt --all -- --check`、wasm check、
  `./scripts/verify.sh`: すべて成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 下書き件数を status に表示するか（現在は positions / histories のみ）。
- 下書きの明示的復元 UI（ADR-0164 / 0157 から継続）。
- 改ざん検知（署名／チェックサム）。
- localStorage 全体 quota の合計管理、1 MiB 超下書きの prune。
- 平文本文を含むバックアップの利用者への注意喚起。
