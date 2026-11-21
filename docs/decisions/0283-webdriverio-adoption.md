---
type: Decision
title: "ADR-0283: WebdriverIO による実 WebView E2E の導入"
description: "Tauri 公式推奨の WebdriverIO と @wdio/tauri-service を debug 限定で導入し、xdotool に依存しない実 WebView 検証を確立する決定。"
tags:
  - ADR
  - E2E
  - WebDriver
generated:
  by: dsh
  at: "2026-11-21T20:58:00+09:00"
status: accepted
---

## 背景

Wayland/Xwayland 環境では `xdotool` のクリックが WebView に届かず、
window 指定のイベントは `XSendEvent` の `send_event=true` を理由に
GTK/WebKitGTK へ拒否され得る。
Tauri 公式は WebdriverIO と `@wdio/tauri-service` を推奨しており、
Linux では埋め込み WebDriver provider と `tauri-driver` が使える。

## 決定

- `@wdio/tauri-service` 1.5.1 と WebdriverIO 9.31 系を devDependency として導入する。
- Rust plugin `tauri-plugin-wdio` / `tauri-plugin-wdio-webdriver` は
  `wdio` feature かつ debug ビルド時のみ登録する。
- `wdio` feature 有効時のみ `desktop/capabilities/wdio` を読み込み、
  既定ビルドと release には WDIO capability を混入させない。
- 既存 CDP/mock E2E 256 件・check 名・
  `check-e2e-count-consistency.sh` は変更しない。
- WDIO は `scripts/verify.sh` の末尾で独立ゲートとして実行する。
- 実 WebView の settings 画面で、editable 状態、updater 固定文言、
  `window.__TAURI__.core.invoke('get_config')` 成功を検証する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| WebdriverIO + `@wdio/tauri-service`（採用） | Tauri 公式推奨で X11 入力に依存しない |
| xdotool / kdotool の併用 | Wayland で入力が不安定なため却下 |
| 既存 CDP/mock E2E のみ | 実 WebView の代替にならないため却下 |
| ydotool | `/dev/uinput` の root 権限が必要で CI に不向き |

## 影響

- `npm install` が必要になる。`npm audit --omit=dev` は 0 件だが、
  devDependency 全体では 25 件の警告が残る。
- WDIO 実行には Node、trunk、DISPLAY、X11 バックエンドが必要。
- `cargo tauri build --debug --features wdio` は wasm-opt の都合で失敗したため、
  現状は trunk dev server を起動して debug バイナリを検証する。
- バンドル済みバイナリは `/settings` の SPA fallback を持たないため、
  WDIO からは dev server の完全 URL を開く。
- `@wdio/tauri-service` の `browser.tauri.execute` は
  "Tauri core.invoke not available" でタイムアウトしたため、
  テストでは WebView 内の `window.__TAURI__.core.invoke` を直接評価する。

## 検証（Runtime Proof）

- `cargo check -p openrustbooks-desktop`: rc=0。
- `cargo check -p openrustbooks-desktop --features wdio`: rc=0。
- `npx tsc -p tsconfig.json`: rc=0。
- `bash scripts/ci-wdio-test.sh`: rc=0。
- `npm run test:wdio`: **1 passing**。
- `./scripts/verify.sh`: 全ゲート成功、既存 E2E **256 passed / 0 failed**、
  `wdio` gate rc=0。

## 未決定

- GitHub Actions の `extra_gates` への WDIO 組み込み
  （共通 workflow 側の Node / DISPLAY / driver 対応が必要）。
- バンドル済み debug バイナリでの WDIO 実行
  （現状は trunk dev server 経由）。
- `browser.tauri.execute` が使える環境の切り分け。
- devDependency の audit 警告 25 件の個別対応。
