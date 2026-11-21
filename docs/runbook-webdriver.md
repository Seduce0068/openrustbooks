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

 1. 依存をロックファイルどおりにインストールする。

```bash
npm ci
```

 `npm ci` は `package-lock.json` と `package.json` の不一致時に失敗する
 （fail-closed）。lock を暗黙に書き換える `npm install` は使わない。
 WDIO ゲートも `npm ci` 済みを前提とし、`npm ls --depth=0` で検査する。

 1. TypeScript と WDIO 設定を型チェックする。

```bash
npx tsc --noEmit
```

 1. WDIO ゲートを実行する。

```bash
bash scripts/ci-wdio-test.sh
```

この script は次を自動で行う。

- `git diff --exit-code HEAD --` による Cargo.lock / package-lock.json の
  不変確認（ステージ済み差分も検知）
- `npm ls --depth=0` による依存整合の検査（`npm ci` が前提）
- `cargo build -p openrustbooks-desktop --features wdio --locked`
- port 1420 の事前占有検査（ビルド前と trunk 起動前の 2 回）
- port 1420 で `trunk serve`（専用プロセスグループ、一時ログ）
- readiness 時に listener が自プロセスグループに属することの確認
- `npm run test:wdio`
- 自プロセスグループのみ停止し、一時ログは失敗時のみ表示して削除
  （PGID 取得失敗時は起動した launcher を停止）

 1. 全体検証に含める。

```bash
export PATH="/tmp/wbbin:$HOME/.cache/trunk/wasm-bindgen-0.2.129:$PATH"
./scripts/verify.sh
```

## バージョン固定ゲート

再現性のため、次は固定値で検証し、逸脱時は fail-closed とする。

- `wasm-pack`: `scripts/ci-wasm-test.sh` が `wasm-pack --version` の出力が
  `wasm-pack 0.13.1` と一致するか、直後に空白があって末尾情報が続く
  場合のみ許可する。`0.13.10` のように数値が続く場合は拒否する。
  未導入時のみ `cargo install wasm-pack --locked --version 0.13.1` で
  導入し、導入後も同じ確認を再実行する。PATH は `CARGO_HOME`
  （既定は `$HOME/.cargo`）配下の `bin` も対象とする。
- `Cargo.lock` / `package-lock.json`: `scripts/ci-wdio-test.sh` が
  `git diff --exit-code HEAD -- <file>` でビルド前後の不変を確認する。
  ステージ済み差分も検知し、ビルド中の変更も失敗する。
- npm: `npm ci` が lock ファイルどおりに依存を導入する。
  `npm install` による lock の暗黙更新は行わない。ゲート内では
  `npm ls --depth=0` で依存ツリーの整合も検査する。

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
- port 1420 が既に占有されている場合、WDIO ゲートはビルド前と
  trunk 起動前の検査で失敗する。既存 listener を kill しないため、
  競合時は占有プロセスを利用側で終了させる。curl が 200 を返しても
  listener が自プロセスグループでなければ ready と判定しない。
- trunk のログは `mktemp -d` 配下に一時保存し、失敗時のみ標準エラーへ
  出力してから削除する。成功時は内容を表示しない。
- GitHub Actions への組み込みは未実施。共通 workflow 側の
  Node / DISPLAY / driver 対応が必要。
