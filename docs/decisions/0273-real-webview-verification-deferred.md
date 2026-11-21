---
type: Decision
title: "ADR-0273: 実 WebView 検証の環境ブロッカーとしての記録"
description: "Tauri CLI 未導入を環境ブロッカーとして記録した決定（その後 ADR-0282 / 0283 で解消）。"
tags:
  - ADR
  - tauri
  - webview
  - verification
generated:
  by: dsh
  at: "2026-11-20T06:14:38+09:00"
status: accepted
---

## 背景

ADR-0272 で desktop の設定保存 UI を完全な `ConfigView` DTO 往復で接続し、
Tauri mock E2E 256/0（check 239 + equal 17）を確認した。
しかし ADR-0253 の実装前提 6 である実 Tauri WebView での往路／復路／表示／a11y 検証は未実施だった。

現状を `Runtime Proof` で確認した。

```text
$ cargo tauri --version
error: no such command: `tauri`
help: find a package to install `tauri` with `cargo search cargo-tauri`
# 終了コード 101

$ echo "$DISPLAY"
:1

$ command -v xdotool
/usr/bin/xdotool

$ command -v wmctrl
/usr/bin/wmctrl
```

すなわち画面操作性（DISPLAY / xdotool / wmctrl）は存在するが、
`cargo-tauri` が未導入のため `cargo tauri dev` / `build` を起動できない。

## 決定

- 実 WebView 検証は**環境ブロッカー**として記録し、
  未実施のまま pass / 完了と記録しない。
- CDP 接続の E2E と Tauri mock E2E は実 WebView の代替ではない。
  これらは JS / IPC 契約の検証であり、OS WebView の描画・IPC 注入・
  スクリーンリーダー連携を証明しない。
- Tauri CLI 導入後に本章の検証手順を実行し、
  結果が出るまで ADR-0272 の「未決定」を追跡先とする。
- 今回は Tauri CLI のインストールと外部ダウンロードを行わない。
- E2E 256 件・check 名・コードは変更しない（文書のみの変更）。

## 検証手順（Tauri CLI 導入後）

1. Tauri CLI を導入し、`cargo tauri --version` が成功することを確認する。
2. `cargo tauri dev` を起動し、desktop ウィンドウが表示されることを確認する。
3. `/settings` を表示し、ブラウザ版の読み取り専用説明ではなく
   desktop の editable フォームになっていることを確認する。
4. `get_config` が成功し、`ConfigView` の全フィールドと `revision` が表示されることを確認する。
5. Enable AI（`settings-ai-enabled`）等の入力を変更し、
   `aria-invalid` / `aria-describedby` の挙動を確認する。
6. 保存ボタンを押し、固定成功文言が `role="status" aria-live="polite"` で告知されることを確認する。
7. `config.toml` が更新され、変更値と `revision` が永続化されていることを確認する。
8. アプリを再起動して `/settings` を再表示し、変更値が復元されることを確認する。
9. スクリーンリーダーとキーボード操作で、ラベル関連付け・フォーカス順序・
   状態告知を確認する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 環境ブロッカーとして記録し未実施を明示（採用） | 未検証を pass とせず、導入後の再現手順を残せる |
| Tauri mock E2E を実 WebView の代替として pass にする | 描画・注入・a11y を証明せず、契約違反 |
| 今回 Tauri CLI を導入して検証する | 外部ダウンロードを禁ずる契約（GOAL.md）に反する |

根拠となる決定:

- [ADR-0253](0253-tauri-settings-save-design.md) — 段階設計と実装前提 6（実 WebView 検証）。
- [ADR-0264](0264-tauri-ipc-path-acceptance.md) — IPC 経路と実 WebView 往路検証の未決定。
- [ADR-0268](0268-csp-consistency-gate.md) — CSP 静的ゲート。実 WebView の CSP 挙動は未検証。
- [ADR-0270](0270-tauri-global-ipc-path.md) — `__TAURI__.core.invoke` 経路の選択。
- [ADR-0272](0272-settings-save-ui.md) — UI 接続完了と実 WebView 未検証の切り分け。

## 未決定

- Tauri CLI の導入責任者と時期。
- GTK / WebKitGTK 依存の有無と導入方法。
- 実 WebView 固有の CSP 挙動（`'unsafe-inline'` 残存と nonce/hash 化）。
- Tauri 更新時の `__TAURI__` / `core.invoke` 形状の変化検知手順。

## 検証（Runtime Proof）

- `cargo tauri --version`: 終了コード 101、`no such command: tauri`。
- `DISPLAY=:1`、`xdotool` は `/usr/bin/xdotool`、`wmctrl` は `/usr/bin/wmctrl`。
- コード・E2E・check 名は未変更。E2E 256 件の既存結果を維持。

## 追記（2026-11-22: 環境ブロッカーの解消）

上の Runtime Proof は決定時点（2026-11-20）の記録であり、現行状態ではない。
Tauri CLI 2.12.5 が導入され、環境ブロッカーは解消した。

- `cargo tauri --version`: `tauri-cli 2.12.5`、rc=0（ADR-0282）。
- 実 Tauri WebView で起動、settings 表示、`get_config` / `update_config_patch`、
  `config.toml` の永続化を確認（ADR-0282）。
- WebdriverIO による独立 E2E ゲートを追加し、運用手順を
  [runbook-webdriver.md](../runbook-webdriver.md) に記録（ADR-0283）。

「未決定」節のうち Tauri CLI の導入と GTK / WebKitGTK 依存は実起動で解消した。
実 WebView の CSP 挙動と `__TAURI__` 形状の更新追従は ADR-0282 / ADR-0283 を追跡先とする。
現行状態は ADR-0282 / ADR-0283 を正とし、本 ADR は履歴として残す。
