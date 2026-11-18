---
type: Decision
title: "ADR-0143: CDP 接続先の決定的照合"
description: "Chrome の browser-level WebSocket を使って E2E が起動した Chrome に確実に接続する決定を記録する。"
tags:
  - ADR
  - testing
  - e2e
generated:
  by: dsh
  at: "2026-11-18T15:00:00+09:00"
status: accepted
---

## 背景

`tools/verify-web-reader.py` は `/json` の `pages[0]` に無条件接続していた
（ADR-0090 / 0142 の未決定）。固定 DEBUG_PORT=9222 に既存 Chrome が残っている場合、
誤ったページへ接続して起動待ちが 20 秒で失敗する可能性があった。

## 決定

- Chrome の **stderr** を PIPE 化し、専用スレッドで
  `DevTools listening on ws://.../devtools/browser/<id>` を捕捉する
  （実機確認: この行は stdout ではなく stderr に出力）。
- 捕捉した browser-level WebSocket に接続し `Target.getTargets` を実行。
  自 Chrome 配下の page target のみを候補化し、
  `targetId` から page WS URL を導出する
  （実機確認: `targetInfos` に `webSocketDebuggerUrl` は無い）。
- 起動前に `/json` で採取した target id set は候補優先順の「ヒント」に格下げ。
- **fail-closed**: browser WS 捕捉タイムアウト 10 秒、
  target 選択 deadline 8 秒、候補 0 件／複数件のすべてで `RuntimeError`。
- E2E 期待総数 229、チェック内容は維持。新規依存なし（Python 標準ライブラリのみ）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| browser-level WS ＋ Target.getTargets（採用） | 起動プロセス固有の ID で決定的に照合できる |
| `/json` の差分方式のみ | 既存 Chrome の page 開閉と競合し得る（監査指摘） |
| `/json/version` の Browser ID | 固定ポート経由で既存側の ID を返し得る |
| 現状維持（pages[0]） | 誤接続リスクが残る |

## 影響

- 既存 Chrome が 9222(IPv4) を占有していても、起動 Chrome は
  `[::1]:9222` にフォールバックし、stderr の行は自プロセスのものを指す
  （実機確認）。
- 起動前に `/json` を読む処理は残るが、接続先選択には使わない。
- `tools/verify-web-reader.py` のみ変更（+124/-8）。

## 検証（Runtime Proof）

- 実機プローブ: stderr 形式、IPv4 占有時の IPv6 フォールバック、
  browser WS → `Target.getTargets` → page WS 導出を確認。
- **ネガティブテスト**: デコイ Chrome を先に起動し
  `window.__E2E_DECOY_MARKER='DECOY-ALIVE-9f3a2c'` を設定 → E2E は 229/229 成功、
  デコイの marker と `about:blank` は不変（誤接続なし＝PASS）。
- **fail-closed 検証**: DevTools 行を出さないフェイク Chrome を
  `--chrome` で指定 → 11 秒後に RuntimeError（exit 1）。
- 5 回連続すべて 229/0（接続先 target id は毎回異なる新規 ID）。
- `./scripts/verify.sh` rc=0（全ゲート・audit・wasm-test・E2E 229/0）。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 5 回連続は回帰確認であり誤接続防止の証明ではない
  （証明はデコイネガティブテストと fail-closed 検証による）。
- `Cdp.send()` の recv タイムアウト（ADR-0142 から継続）。
- リトライ回数・バックオフ、`localStorage.clear()` のタイミング（ADR-0142 から継続）。
- 固定ポート変更の再評価、20 回連続実行、CLS 計測、5 ブラウザ実機検証
  （ADR-0140 / 0142 から継続）。
