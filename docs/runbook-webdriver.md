---
type: Playbook
title: "WebdriverIO による実 WebView E2E の運用"
description: "openrustbooks の実 Tauri WebView を WebdriverIO で検証する手順と制約。"
tags:
  - playbook
  - e2e
  - webdriver
generated:
  by: dsh
  at: "2026-11-21T21:00:00+09:00"
status: accepted
---

## 目的

`xdotool` の Wayland/Xwayland 入力に依存せず、Tauri の実 WebView で
設定画面の描画と IPC を検証する。既存 CDP/mock E2E の代替ではなく、
ADR-0282 / ADR-0283 の補完ゲートとして使う。

## 前提

- Node 24 系、npm 11 系。
- `trunk` と `wasm-bindgen` が PATH にあること。
- `DISPLAY` が設定され、X11 バックエンドが利用できること。
- `cargo tauri --version` が成功すること（現状は 2.12.5）。
- `RUST_LOG` は desktop では tracing filter として効かない（ADR-0282）。

## 手順

1. 依存をインストールする。

```bash
npm install
```

1. TypeScript と WDIO 設定を型チェックする。

```bash
npx tsc --noEmit
```

1. WDIO ゲートを実行する。

```bash
bash scripts/ci-wdio-test.sh
```

この script は次を自動で行う。

- `cargo build -p openrustbooks-desktop --features wdio`
- port 1420 で `trunk serve`
- `npm run test:wdio`
- trunk の停止と port 1420 の listener 回収

1. 全体検証に含める。

```bash
export PATH="/tmp/wbbin:$HOME/.cache/trunk/wasm-bindgen-0.2.129:$PATH"
./scripts/verify.sh
```

## 検証内容

`e2e/specs/settings.e2e.ts` は次を確認する。

- dev server の `/settings` に遷移できる
- `h1` が `Settings` である
- `#settings-revision` が表示される
- `#settings-language` が表示され、editable である
- 「更新確認: 未配線」が DOM に含まれる
- `window.__TAURI__.core.invoke('get_config')` が成功し、数値 revision を返す

## 注意点

- `browser.tauri.execute` は現在の環境で
  "Tauri core.invoke not available after 5s timeout" になる。
  テストは WebView 内の `__TAURI__` を直接評価する。
- バンドル済みバイナリは `/settings` の SPA fallback を持たないため、
  `tauri://localhost/settings` は `asset not found` になる。
- `cargo tauri build --debug --features wdio` は wasm-opt で失敗したため、
  現状は trunk dev server 経由で実行する。
- WDIO service の window-state hook は warning を出すが、
  テスト本体は成功する。warning を根拠に失敗扱いしない。
- `npm audit --omit=dev` は 0 件。devDependency 全体の 25 件は既知。
- GitHub Actions への組み込みは未実施。共通 workflow 側の
  Node / DISPLAY / driver 対応が必要。
