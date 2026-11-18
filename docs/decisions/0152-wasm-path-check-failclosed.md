---
type: Decision
title: "ADR-0152: WASM パス検査の fail-closed 化"
description: "配布 WASM の開発ホスト絶対パス検査を、glob 非マッチと grep エラーでも失敗するようにする決定を記録する。"
tags:
  - ADR
  - tooling
  - security
generated:
  by: dsh
  at: "2026-11-18T19:50:00+09:00"
status: accepted
---

## 背景

旧 `scripts/verify.sh` の検査は
`grep -a -q '/home/' "$DIST_DIR"/*_bg.wasm` だった。
glob が非マッチなら `grep` は rc=2 を返すが `if` 条件では偽扱いとなり、
パス検査が黙って成功する fail-open 状態だった（ADR-0151）。
変換対象も `*_bg.wasm` のみで、生成 JS グルーコードは検査していなかった。

## 決定

- `HOME` を `${HOME%/}` で正規化し、空文字なら即失敗する。
  remap フラグ（`--remap-path-prefix`）と grep パターンの両方で同じ値を使う。
- `nullglob` の元状態を保存・復元しつつ、
  `"$DIST_DIR"/*_bg.wasm` と `"$DIST_DIR"/*.js` を配列へ展開する。
- いずれかの配列が 0 件なら即失敗（fail-closed）。
- `grep -a -q -F -- "$home/" "${files[@]}"` で固定文字列検査する。
  - rc=1（一致なし）は成功。
  - rc>=2（ファイル読取不可等）は「検査不能」として失敗。
- メッセージは WASM と JS グルーを含む旨を明記。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| nullglob＋配列件数＋rc 分類（採用） | 既存の意図を保ちつつ fail-closed |
| 旧 `if grep` のまま | glob 非マッチと rc=2 が fail-open |
| `/home/` 固定継続 | HOME が別配下だと検出できない |

## 影響

- trunk build 直後のパス検査で、WASM / JS の両方が走査される。
- 検査失敗時は E2E が実行されない（既存の設計どおり）。
- ADR-0088:26 の「`/home/` が含まれていれば失敗」は本 ADR で上書きする。
- 新規依存なし。

## 検証（Runtime Proof）

- `bash -n scripts/verify.sh`、`shellcheck`、`shfmt -d` すべて成功。
- 負テスト:
  - 空 dist → rc=1（「配布 WASM が見つかりません」）。
  - 正常 dist（wasm + js）→ rc=0。
  - JS に `$HOME/` を含む → rc=1。
- `./scripts/verify.sh` rc=0、E2E **231 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- CI の `extra_gates` へ同じ検査を載せるか（共通ワークフロー変更が必要）。
- 対象を全 `*.wasm` や `.js.map` へ拡げるか。
- 検査を verify.sh の定常テストとして切り出すか。
- 文字列中の偶然一致を誤検知として許容する判断の再評価。
