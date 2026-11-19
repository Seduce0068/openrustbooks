---
type: Decision
title: "ADR-0265: E2E のブラウザ横断実行"
description: "現行 E2E 255 件を Chrome / Edge で実機実行し、他ブラウザの環境ブロッカーを記録する決定。"
tags:
  - ADR
  - e2e
  - browser
generated:
  by: dsh
  at: "2026-11-20T03:40:00+09:00"
status: accepted
---

## 背景

グローバル規約の対象ブラウザは Chrome / Edge / Brave / Firefox / Safari。
`tools/verify-web-reader.py` は CDP（`--chrome` で実行ファイル指定）で
単一エンジンを起動し、check 名 255 件（check 238 + equal 17）を検証する
（[ADR-0263](0263-e2e-count-gate.md)）。
従来の E2E は Chrome / Chromium のみで、5 ブラウザ検証は未決定だった
（[ADR-0232](0232-initial-release-scope.md) / [ADR-0224](0224-todo-reconciliation.md)）。

## 決定

- 現行ソースから新規 dist をビルドし、Chrome と Edge で同一 255 check を実行する。
- check 名・件数・E2E スクリプトは変更しない（255 維持）。
- Firefox / Brave / Safari は pass / n/a とせず、
  環境ブロッカーとして記録する。
- 結果は ADR の Runtime Proof とし、今後のブラウザ追加検証の基準とする。

## 根拠

- E2E のブラウザ選択は `--chrome` 実行ファイル 1 つ（既定 `google-chrome`）。
- CDP 接続は Chromium 固有（browser-level WebSocket + `Target.getTargets`）。
- `DOM.setFileInputFiles` / `Page.setDownloadBehavior` なども CDP 依存。
- Firefox は RDP（CDP ではない）、Safari / WebKit は独自プロトコルで、
  同一スクリプトからの駆動には別ドライバ層が必要。
- 出典: VC-015（[requirements.md](../requirements.md)）、
  [ADR-0090](0090-e2e-expected-count.md)（コード出典）、
  [ADR-0262](0262-e2e-count-documentation-refresh.md) / [ADR-0263](0263-e2e-count-gate.md)（255 維持）。

## 影響

- Chrome / Edge で 255/0 の実機結果を得た（下記 Runtime Proof）。
- Brave は未インストール、Safari は Linux ホストで起動不可、
  Firefox はプロトコル不一致のため未対応。
- 対象 5 ブラウザの要件は維持し、未達 3 件は環境ブロッカーとして追跡する。

## 検証（Runtime Proof）

- 日時: 2026-11-20（JST）。
- ビルド: `trunk build --offline --locked --dist /tmp/orb-cross-dist` rc=0。
- Chrome: `Google Chrome 158.0.8127.36`。
  E2E は `--dist /tmp/orb-cross-dist --chrome /usr/bin/google-chrome`
  `--port 18099 --debug-port 19222` で実行し、
  **passed: 255, failed: 0 / all checks passed**、
  `ok` 行数 255、rc=0（`/tmp/e2e-chrome.log`）。
- Edge: `Microsoft Edge 157.0.4341.60`、
  同 dist を `--chrome /usr/bin/microsoft-edge --port 18100 --debug-port 19223` で実行
  → **passed: 255, failed: 0 / all checks passed**、
  `ok` 行数 255、rc=0（`/tmp/e2e-edge.log`）。
- Firefox: `Mozilla Firefox 160.0.1` は installed だが CDP 非対応のため未実行。
- Brave / Safari: `command -v` で not_found。Safari は macOS 専用。
- `./scripts/verify.sh`: 全ゲート成功、E2E 255 passed / 0 failed。

## 未決定

- Brave のインストール責任者・期限と、Safari 検証用 macOS 環境の用意。
- Firefox / Safari を動かす WebDriver BiDi / Playwright 等のドライバ選定。
- エンジン依存アサーション（TouchEvent / CLS / forced-colors / ダウンロード名）の扱い
  （件数維持で同等 check に置換するか、既知ベースラインとするか）。
- manifest への `browser executable` / `browser version` / `cdp port` 記録（ADR-0219 と整合）。
- 固定ポート 9222 / 8099 の衝突時 fail-closed 運用、固定タイムアウトの他エンジンでの妥当性。
