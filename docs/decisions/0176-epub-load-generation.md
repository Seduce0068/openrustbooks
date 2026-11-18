---
type: Decision
title: "ADR-0176: EPUB ロード世代の分離"
description: "非同期 EPUB ロードに独立した世代カウンタを導入し、古いロード完了による状態上書きを防ぐ決定を記録する。"
tags:
  - ADR
  - web
  - concurrency
generated:
  by: dsh
  at: "2026-11-19T02:35:00+09:00"
status: accepted
---

## 背景

`on_file_change()` はファイル選択ごとに `spawn_local` で EPUB をロードしていたが、
完了前に別ファイルを選んでも古いタスクをキャンセルできなかった。
古いタスクが後で完了すると `book` / `history` / `chapters` / `cover` /
`status` / `render_chapter` を更新し、最後に選んだ本が上書きされる
（R6: PR-H03 / TD-H03）。

## 決定

- `load_generation: Rc<Cell<u64>>` を描画用 `current_generation`（ADR-0053 / 0054）
  とは**完全に別カウンタ**として導入する。
  - 描画用は章移動／位置適用の破棄を担当。
  - ロード用はファイル選択の古い完了の破棄を担当。
- インクリメントは input / files / size チェック通過後、
  `position_key()` 計算前に行う。無効な change で進行中ロードを破棄しないため。
- 値は `load_gen` として `spawn_local` へ move capture し、
  完了時に `load_gen != load_generation.get()` なら即 return する。
- ガード対象（計 10 箇所）:
  async 先頭、`array_buffer()` エラー分岐・成功直後、
  `WasmEpubBook::new()` エラー分岐・成功直後、0 章、
  `chapter_href` エラー分岐、UI state 更新前（cover object URL 生成前）、
  history / book 適用前、`render_chapter()` 呼び出し前。
- `u64` を採用（`current_generation` の u32 とは方針を統一しない）。
  実用上オーバーフローしないため `+1` のまま。
- E2E は追加せず **247 維持**。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 独立した load_generation（採用） | 章移動とファイル選択を独立に破棄できる |
| current_generation と統合 | 章移動がロードを誤破棄する |
| タスク cancel | `spawn_local` にキャンセル手段が無い |

## E2E を追加しない理由

- ロード経路の唯一の `await` は `file.array_buffer()` で、
  `WasmEpubBook::new()` / toc / cover / `render_chapter()` は同期。
- CDP の `DOM.setFileInputFiles()` + change dispatch はローカルファイルを
  即渡すため、1 つ目の `array_buffer()` 完了を 2 つ目より確実に遅らせる
  フックが無い。
- 「2 回 dispatch → 2 つ目本文待ち」では 1 つ目の後完了を検出できず
  フレークする。巨大ファイルでの遅延も 256 MiB 上限・実行時間・
  localStorage 後始末の 문제があり、247 契約を壊すリスクが利益を上回る。
- 回帰検知は静的レビューと既存 E2E 247 項目に依存する（未決定）。

## 検証（Runtime Proof）

- `python3 -m py_compile`、`cargo fmt --all -- --check`、
  wasm32 web check、`./scripts/verify.sh`: すべて成功。
- E2E **247 項目すべて成功**。
- 全単体テスト core 210 + desktop 56 + web 2、wasm-test 10、
  audit 脆弱性 0 件。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- E2E での再現手段（File.arrayBuffer の遅延注入フック等）。
- `array_buffer()` は abort できず、古い decompress が無駄に走る性能影響
  （未測定）。
- 同一ファイル再選択で世代を進めるか。
- `current_generation`（u32）と `load_generation`（u64）の型統一。
