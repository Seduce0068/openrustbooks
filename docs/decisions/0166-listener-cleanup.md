---
type: Decision
title: "ADR-0166: イベントリスナーの cleanup"
description: "forget() によるリスナー蓄積を on_cleanup + SendWrapper による解除へ変更する決定を記録する。"
tags:
  - ADR
  - web
  - leptos
generated:
  by: dsh
  at: "2026-11-18T23:35:00+09:00"
status: accepted
---

## 背景

`web/src/lib.rs` の save-error（Editor / Reader）、画像 load、
prefers-color-scheme change の 4 つの Closure が `forget()` で
ページ寿命保持だった。SPA 再マウントでウィンドウ／ドキュメントへ
リスナーが蓄積し、古い signal への set が残る（ADR-0165 の未決定）。

## 決定

- `on_cleanup()`（Leptos 0.8.22）で登録解除する。
- 素直な実装は `on_cleanup` が `FnOnce() + Send + Sync + 'static` を要求し、
  `Closure` / `Window` / `Document` / `MediaQueryList` が `!Send + !Sync` のため
  コンパイルエラー（E0277 × 8、Runtime Proof）。
- よって `leptos::__reexports::send_wrapper::SendWrapper` で
  `Option<(target, event_name, Closure)>` を包み、cleanup 内で
  `take()` → `remove_event_listener_with_*()` → Closure を drop する。
  - Leptos 自身が `window_event_listener_untyped` 内部で使う型で、新規依存なし。
- 対象 4 件:
  - Editor save-error（`window` / `openrustbooks:save-error:editor`）
  - Reader 画像 load（`document` キャプチャ / `load` / `capture=true`）
  - Reader save-error（`window` / `openrustbooks:save-error:reader`）
  - `prefers-color-scheme` change（`MediaQueryList` / `change`）
- `set_downscaled_src()` の load リスナーは `Closure::once_into_js` の
  使い捨てであり、`data-downscaled="1"` ガードが再処理を防ぐため対象外
  （ADR-0148 の未決定と接続）。
- `.forget()` は 0 件になった（既存 key_listener の cleanup は維持）。

## 影響

- 登録 API・イベント名・通知文言・挙動は不変。
- `leptos::__reexports::send_wrapper` は `#[doc(hidden)]` の re-export。
  Leptos 0.8.22 固定では問題ないが、アップグレード時にパス確認が必要
  （未決定）。

## 検証（Runtime Proof）

- `cargo fmt --all -- --check`、
  `cargo check --all-targets --locked`、
  `cargo clippy --all-targets --locked -- -D warnings`、
  wasm32 core / web check、wasm-pack test --node（10 件）、
  `cargo audit --no-fetch`、trunk build / wasm パス検査: すべて成功。
- E2E **241 項目すべて成功**。
- `grep -c '\.forget()' web/src/lib.rs` = 0。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- SPA 再マウント時の cleanup 発火そのものの直接 E2E 検証
  （現状は Page.navigate ベースで未検証）。
- `send_wrapper` の re-export パスが変わった場合の代替
  （直接 gloo-events の `EventListener` を SendWrapper で包む案）。
- `set_downscaled_src` の 2 回呼び出し時に JS exception が出る場合の扱い
  （`data-downscaled` ガードが前提、ADR-0148 から継続）。
