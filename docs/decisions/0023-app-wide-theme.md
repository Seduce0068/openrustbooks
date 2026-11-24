---
type: Decision
title: "ADR-0023: アプリ全体のダークテーマ"
description: "リーダーのダーク表示トグルをアプリ全体（ツールバー・一覧画面）へ拡張する決定を記録する。"
tags:
  - ADR
  - web
  - wasm
  - accessibility
generated:
  by: dsh
  at: "2026-11-15T13:08:00+09:00"
status: accepted
---

## 背景

ADR-0018 まででダーク表示は `.reader-content` にしか適用されず、ツールバー
や一覧画面はライトのままだった（ADR-0018 の未決定「アプリ全体のテーマ
（ツールバー・一覧画面を含む）」）。手動トグルを `<html>` のクラスへ反映し、
CSS 変数を切り替える。

## 決定

### トグル

- `Dark` / `Light` ボタンで `<html>` に `dark` クラスを付け外しする。
- 設定は localStorage `openrustbooks:settings` の `dark` に保存する（既存）。
- 起動時に保存値を読んで `<html>` のクラスへ反映する（`Effect`）。
- `prefers-color-scheme: dark` の既定は維持する（手動トグルが無い場合）。

### CSS

- `html.dark` で `--color-bg` / `--color-fg` / `--color-muted` /
  `--color-border` / `--color-primary` / `--color-secondary` をダーク値へ
  上書きする。
- `.reader-content` の個別ダークスタイルも `html.dark` に統合する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| `<html>` のクラス + CSS 変数（採用） | 既存の `prefers-color-scheme` と併存でき、追加依存なし |
| JavaScript で各要素の style を書き換える | 再描画が複雑で、動的追加に追随できない |
| OS の設定にだけ追従 | 手動切り替えができない |

## 影響

- ダーク表示がツールバー・一覧画面・本文全体に適用される。
- 設定は localStorage に保存され、外部送信は行わない。
- `prefers-color-scheme` の既定は維持される。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 97 項目が
  3 回連続で成功。アプリ全体のテーマについて、`Dark` で `<html>` に `dark`
  クラスが付くこと、ツールバーの背景がダークになること、リロード後も
  維持されること、`Light` で外れることを確認した。
- `cargo test --all-targets --locked` は 119 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI（`Seduce0068/workflows/rust-gates.yml@v1`）が成功する。

## 未決定

- EPUB 内 CSS を適用する場合との優先順位（ADR-0018 から継続）。
- 設定のエクスポート・インポート（ADR-0018 から継続）。
- 縦書き・ページ送り（ADR-0012 から継続）。
- OS のカラースキーム変更への自動追従。
