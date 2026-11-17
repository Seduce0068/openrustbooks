---
type: Decision
title: "ADR-0033: OS のカラースキームへの追従"
description: "prefers-color-scheme を検知してダーク表示を自動で切り替える決定を記録する。"
tags:
  - ADR
  - epub
  - web
  - wasm
generated:
  by: dsh
  at: "2026-11-15T21:20:00+09:00"
status: accepted
---

## 背景

ADR-0018・ADR-0023 まででダーク表示は手動トグルのみだった。OS がダーク
モードの環境では、起動のたびに手動で切り替える必要があり、読書開始時の
まぶしさを招く（ADR-0018・ADR-0023 の未決定「テーマの自動切替
（prefers-color-scheme）」）。

## 決定

### 追従モード

- 表示設定に `followSystem`（追従）を追加する。
- ツールバーに `<button id="reader-theme-auto" aria-pressed="...">` を置く。
  ラベルは `Auto:on` / `Auto:off` で状態を示す。
- 追従が有効なときは `prefers-color-scheme: dark` を `matchMedia` で
  評価し、その結果を実際のダーク表示に使う。
- `matchMedia` の `change` イベントを購読し、OS の設定変更にリアルタイムで
  追従する。
- 手動トグル（`#reader-theme`）を押すと追従は解除され、手動の値が優先される。

### 実際に適用する値の集約

「実際に適用するダーク表示」を `Memo` に集約し、`<html>` の `dark` クラスと
リーダーのコンテナの `reader-dark` クラスの両方がこの 1 つの値を見る。
追従の有無による分岐を UI の各所に散らさない。

### 永続化

追従の選択は既存の表示設定と同じ localStorage のキー
（`openrustbooks:settings`）に `followSystem` として保存し、再読み込み後に
復元する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| `matchMedia` + `change` 購読（採用） | 追加依存なし、リアルタイム追従できる |
| CSS の `@media (prefers-color-scheme)` だけに任せる | 手動選択との優先順位を制御できない |
| 起動時だけ判定する | OS の設定変更に追従できない |

## 影響

- OS がダークモードなら、追従を有効にしているあいだ自動でダーク表示になる。
- 手動トグルは従来どおり使える（押すと追従が解除される）。
- 表示設定の保存形式にキーが 1 つ増える。既存の設定 JSON にキーが無い
  場合は `false`（追従しない）として読むため、後方互換がある。
- 追加依存は無く、既存の `web-sys` 機能の範囲で実装している。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 129 項目が
  3 回連続で成功。追従について、追従ボタンの存在、押下状態、OS がダーク
  ならダーク表示になること、追従の選択が保存されること、OS がライトなら
  ライト表示になること、手動切替で追従が解除されることを
  `Emulation.setEmulatedMedia` で OS 設定を切り替えて確認した。
- `cargo test --all-targets --locked` は 119 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 解決済み

- 追従の初期値（OS がダークなら初回から追従を有効にするか）: ADR-0038 で
  初回起動時に OS の設定を初期値として採用した。
- `forced-colors` への対応: ADR-0039 で `@media (forced-colors: active)` の
  スタイルを追加した（`prefers-contrast` は ADR-0039 で対象外と判断）。

## 未決定

- `matchMedia` の購読解除（ページ寿命まで保持しているので不要だが、
  将来 Reader が再マウントされる構成になった場合は必要）。
- `prefers-contrast: more` への対応（ADR-0039 から継続）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
