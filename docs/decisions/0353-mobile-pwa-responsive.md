---
type: Decision
title: "ADR-0353: PWA とレスポンシブ UI によるモバイル対応"
description: "iOS は PWA を各自インストールし、Android は別途ネイティブ版を提供する方針で 320px・タッチ対応 UI を実装する決定。"
tags:
  - ADR
  - mobile
  - PWA
  - accessibility
generated:
  by: dsh
  at: "2026-11-23T16:12:00+09:00"
status: accepted
---

## 背景

要件書にはスマホ利用範囲の课题 (VC-011 / 優先して具体化する事項) はあったが、
正式な REQ が無かった。ユーザー指示により REQ-011 を追加した:
Android はアプリ版、iOS はネイティブ版を作らず PWA を各自インストールする方式。

## 決定

- 小画面は 320 CSS px までを対象とし、横方向のページスクロールを発生させない。
- `@media (pointer: coarse)` で操作領域を 44×44 CSS px 以上にする。
- `dvh` / `minmax(0, 1fr)` / `flex-wrap` / `overflow-wrap` を用いて
  header / toolbar / reader / editor / settings が狭幅で壊れないようにする。
- `viewport-fit=cover` と `env(safe-area-inset-*)` でノッチを回避する。
- Web app manifest に安定 `id` / `start_url` / `display: standalone` / 192・512 アイコンを設定する。
- Web Install API が使えるブラウザでのみインストールボタンを表示し、
  iOS Safari では About に「共有 → ホーム画面に追加」の固定案内を表示する。
- service worker は同一起点のみ、versioned cache と旧 cache 削除を行う。
- E2E は CDP emulation で 320px / coarse pointer / reader / editor / safe-area を検証する。

## 検証（Runtime Proof）

- E2E 263 件 → **273 件（check 256 + equal 17）**。新規 10 check:
  PWA ホーム、manifest link、theme-color、SW 登録、install 隠蔽、
  320px 横スクロール、coarse 44px、About iOS 案内、reader 320px、editor 320px。
- `python3 tools/verify-web-reader.py --dist web/dist`: **passed: 273, failed: 0**。
- `cargo fmt --all -- --check`: rc=0。
- `cargo check -p openrustbooks-web --target wasm32-unknown-unknown --locked`: rc=0。
- `cargo test -p openrustbooks-web --locked`: 31 passed / 0 failed。
- `npx tsc --noEmit`: rc=0。
- `scripts/check-e2e-count-consistency.sh`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **273 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- iOS Safari の実機でのホーム画面追加動作（macOS / iOS 実機が無いため未検証）。
- service worker のキャッシュ名をビルドハッシュへ連動させる恒久対策。
- モバイルでの AI / 設定機能のfull 対応範囲。
