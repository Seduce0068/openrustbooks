---
type: Decision
title: "ADR-0235: E2E 待機の条件待ち化と swipe 二重 dispatch の解消"
description: "swipe の二重 dispatch を 1 回へし、到達条件が明確な待機を条件待ちへ置換する決定。"
tags:
  - ADR
  - testing
  - e2e
  - wait
generated:
  by: dsh
  at: "2026-11-19T18:34:27+09:00"
status: accepted
---

## 目的

E2E（`tools/verify-web-reader.py`）の swipe スタブが同じ TouchEvent を 2 回 dispatch
していた問題と、状態変化を固定時間の `asyncio.sleep()` で決めつけていた待機を解消し、
到達条件が明確な箇所を条件待ちへ統一する。R6 R4-e2e#3,#4 / C08 の契約を満たす。

## 現状

- 旧 `swipe()` は先頭の `cdp.evaluate()` で `arguments[0..2]` を参照する JS 式を渡していた。
  `Cdp.evaluate()` は式文字列のみを受け取り引数を渡さないため、`dx` / `dy` / `touches` が
  未埋め込みの TouchEvent が発火していた。その後 f-string で実引数を埋めた版が同じ
  touchstart / touchend を再 dispatch し、二重発火になっていた。
- `asyncio.sleep()` が多数残り、章移動や DOM 更新・保存完了を固定時間で決めつけていた。
  時間経過は到達条件ではないため、環境差で取りこぼすとフレークの原因になる。
- `wait_until()` は ADR-0183 で `RuntimeError` と一般例外を再送出する fail-closed 契約のまま。
- E2E の check 名 254 件（check 237 + equal 17）と `Checker(expected=254)` は不変。

## 決定

### swipe の単一 dispatch

- 先頭の evaluate を削除し、f-string 版の 1 回だけ dispatch する。
- `dx` / `dy` / `touches` は整数を f-string で埋め込み、既存の swipe 意味を維持する。
- dispatch の同期完了印として `window.__orbSwipeDone` をインクリメントし、
  条件待ちのベースラインに使う。カウンタは check 名・期待総数に影響しない。

### 条件待ちへの置換

- 到達条件が明確な章移動・縦 / マルチタッチ・ダーク切替・ファイル選択後の DOM / 保存・
  リセット後を `wait_until()` / `wait_swipe_done()` / `wait_swipe_ignored()` ベースへ置換する。
- 負向チェック（縦移動・マルチタッチで章が変わらない）は、baseline カウンタと
  第 1 章本文の存在、および 0.3 秒の維持確認で判定する。
- 到達条件のない残りの sleep は据え置く（後述）。

### fail-closed の維持

- `wait_until()` は `RuntimeError` / 一般例外をそのまま再送出し、握り潰さない
  （ADR-0183 / ADR-0144）。
- CDP 接続先照合（ADR-0143）と check 名一意性（ADR-0182）は変更しない。

## 選択肢と根拠

| 選択肢 | 判断 | 根拠 |
| --- | --- | --- |
| 全 sleep を条件待ちへ置換 | 却下 | 残る 106 件は到達条件が明示されておらず、無理な置換はフレーク化と検出力低下を招く。 |
| 固定 sleep を維持 | 却下 | 二重 dispatch が残り、状態変化を時間で決めつけるため環境差に弱い。 |
| 先頭 evaluate のみ削除 | 部分採用 | 二重発火は解消するが状態依存 sleep が残り、本次の目的を満たさない。 |
| 1 回 dispatch ＋明確な箇所だけ条件待ち（採用） | 採用 | 未埋め込みイベントを排除しつつ、観測可能な到達条件だけを決定的な待ちへ置換できる。 |
| wait_until の例外を握り潰す | 却下 | ADR-0183 の fail-closed 契約に反し、基盤失敗がタイムアウトまで隠れる。 |

## 影響

- 変更: `tools/verify-web-reader.py`（実装エージェントが実施）。Rust コード・依存・CI 設定は変更なし。
- 文書: 本 ADR を追加し、decisions / docs の各 index（234 → 235 件）、README、`docs/log.md`、
  `GOAL.md`、`todo.md`、`docs/e2e-traceability.md`（概要へ 1 行）を追従する。
- `docs/e2e-traceability.md` の check 名と件数は一字も変更しない。AST 254/254 を維持する。
- R4-e2e#3,#4 / C08 を「未着手」から「完了」へ移動する。
- E2E 期待総数 254、全ゲート成功は不変。

## 検証（Runtime Proof）

- 実装エージェントの検証: `python3 -m py_compile` OK、AST 抽出 check 237 + equal 17 = 254、
  期待総数 `Checker(expected=254)` と一致、`./scripts/verify.sh` 全ゲート終了コード 0、
  E2E 254/0。
- 文書エージェントの再検証（2026-11-19）: `python3 -m py_compile tools/verify-web-reader.py` OK、
  AST 抽出 237 + 17 = 254 / distinct 254 / `expected=254`、`npx markdownlint-cli2` は 0 issues。
- 再現コマンド（Kubuntu / リポジトリルート）:

```bash
python3 -m py_compile tools/verify-web-reader.py
npx --yes markdownlint-cli2 'docs/**/*.md' README.md todo.md
```

## 未決定

### 残る固定 sleep（106 件）

到達条件がなく、タイマーや描画の完了を直接観測できないため据え置いた。アプリ側の
完了シグナル追加など決定的な条件が用意できた箇所から段階的に置換する。対象の選定と
シグナル設計は未決定。

### CDP dispatch / paint timing

`dispatchEvent()` が保証するのはハンドラ実行の同期完了までで、コンポジタの paint 完了は
CDP 式の DOM 読取りでは観測できない。現状は DOM テキストとカウンタを条件にしており、
視覚上の描画遅延を直接ゲートしていない。スクリーンショット比較等の導入は未決定。

### 5 ブラウザ差

自動 E2E は単一エンジン（CDP 接続の Chrome）のみ。Chrome / Edge / Brave / Firefox /
Safari の 5 ブラウザで TouchEvent のディスパッチ順序や paint タイミングが同一かは未検証
（ADR-0145 / 0159 / 0232 の未決定を継続）。実機検証とゲート化の方法は未決定。

## 参照

- [ADR-0090: E2E 期待 check 総数の検証](0090-e2e-expected-count.md)
- [ADR-0143: CDP 接続先の決定的照合](0143-cdp-target-matching.md)
- [ADR-0144: CDP recv タイムアウト](0144-cdp-recv-timeout.md)
- [ADR-0182: E2E check 名の一意性検証](0182-e2e-unique-check-names.md)
- [ADR-0183: wait_until の例外伝播](0183-wait-until-failclosed.md)
- [ADR-0232: 初版リリーススコープ](0232-initial-release-scope.md)
- [docs/e2e-traceability.md](../e2e-traceability.md)
- [GOAL.md](../../GOAL.md)
