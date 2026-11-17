---
type: Decision
title: "ADR-0090: E2E 期待 check 総数の検証"
description: "未実行の check を見逃さないため、期待総数を宣言して検証する決定を記録する。"
tags:
  - ADR
  - testing
generated:
  by: dsh
  at: "2026-11-17T23:25:00+09:00"
status: accepted
---

## 背景

`checker.check()` は条件付きブロック内でも呼ばれ、DOM ノードが見つからない
場合などには check 自体がスキップされる（todo.md R4-e2e#1 / C06）。
その場合、実 passed 数が静的な check 呼び出し数より少なくても
テストは成功してしまった。

## 決定

- `Checker` に `expected` を追加する。
- `run_checks()` で期待総数を設定する。実装時は 220
  （`checker.check` 203 + `checker.equal` 17）、その後 E2E 追加に伴い
  223 → 228 へ更新された。件数は常に `tools/verify-web-reader.py` の
  `Checker(expected=...)` を出典とする。
- `finish()` で `passed + len(failures) == expected` を検証し、
  乖離があれば「expected N checks, but executed M」を表示して失敗（rc=1）とする。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 期待総数を宣言（採用） | スキップされた check を確実に検出できる |
| 動的に数える | スキップと正常な条件分岐を区別できない |
| 現状維持 | 未実行 check が黙って見逃される |

## 影響

- E2E の check が追加・削除された場合、期待値も更新する必要がある。
- 通常実行では 223 項目すべて成功する。
- プロダクションコードは変更しない。

## 検証

- 実装直後（期待値 220）の通常実行: `passed: 220, failed: 0`、`all checks passed`。
- 期待値を 221 にした一時コピーでは
  「expected 221 checks, but executed 220」を表示し rc=1 になることを実際に確認した。
- その後 E2E の追加に伴い期待値は 223 → 228 へ更新されている（ADR-0091 〜 0095）。
  期待総数は常にコードを出典とし、ADR の記載は作成時点の実績である。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- CDP 接続先を起動 PID と照合し、他 Chrome への誤接続を防ぐ（todo.md C06 の残り）。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
EOF
