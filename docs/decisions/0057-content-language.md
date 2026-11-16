---
type: Decision
title: "ADR-0057: 本文言語の設定"
description: "EPUB の言語を本文要素の lang 属性に反映し、スクリーンリーダーの発音を最適化する決定を記録する。"
tags:
  - ADR
  - epub
  - web
  - accessibility
generated:
  by: dsh
  at: "2026-11-17T06:25:00+09:00"
status: accepted
---

## 背景

ADR-0021 の未決定「スクリーンリーダーでの本文読み上げの最適化」。
`<html lang="ja">` は UI 向けに固定で、EPUB 本文が英語など別言語の場合、
スクリーンリーダーが日本語の発音規則で読み上げていた。core は
`metadata.language` を保持しているため、これを本文要素に反映する。

## 決定

- `WasmEpubBook::language()` を追加し、EPUB の言語（例: `ja`, `en`）を返す。
- 本を開いたときに言語を取得し、本文要素（`.reader-html`）の `lang` 属性に設定する。
- 言語が不明な場合は `lang` を設定せず、UI の `<html lang="ja">` が継承される。
- 章を切り替えても現在の本の言語が維持され、別の本を開くと更新される。
- UI 全体の `<html lang="ja">` は変更しない（UI は日本語固定）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 本文要素に lang を設定（採用） | 本文だけ正しい言語で読み上げられ、UI に影響しない |
| `<html lang>` を変更 | UI の日本語読み上げが壊れる |
| 何もしない | 外国語本文の発音が不自然 |

## 影響

- 外国語の EPUB をスクリーンリーダーで読む際の発音が改善される。
- UI の言語設定は変わらない。
- 追加依存は無く、既存の `web-sys` / wasm-bindgen の範囲で実装している。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 174 項目が
  5 回連続で成功。サンプル EPUB（言語 `ja`）で本文要素に `lang="ja"` が
  付くこと、UI 全体の `<html lang="ja">` が変わらないことを確認した。
- `cargo test --all-targets --locked` は 121 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- ページ見出しへの `aria-labelledby` 設定など、さらに詳細な本文構造の提供。
- `epub:type="noteref"` / `footnote` の明示的な解釈（ADR-0055 から継続）。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
