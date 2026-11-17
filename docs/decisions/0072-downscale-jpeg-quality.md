---
type: Decision
title: "ADR-0072: 縮小 JPEG の品質指定"
description: "画像縮小時の JPEG 再エンコードに品質 0.85 を明示する決定を記録する。"
tags:
  - ADR
  - epub
  - web
  - performance
generated:
  by: dsh
  at: "2026-11-17T10:40:00+09:00"
status: accepted
---

## 背景

ADR-0071 の未決定「JPEG の再エンコード品質（quality）の最適値」。
`to_blob_with_type()` の既定品質はブラウザ依存で、ファイルサイズと画質の
バランスが安定しなかった。

## 決定

- MIME が `image/jpeg` の場合、`{ quality: 0.85 }` を encoder options として
  `canvas.to_blob_with_type_and_encoder_options()` を呼ぶ。
- PNG / WebP は従来どおり `to_blob_with_type()` を使う。
  quality は形式により意味が異なるため指定しない。
- 既定オフ（ADR-0067）、元 MIME 保持（ADR-0071）、表示サイズ不変の方針は維持。
- JPEG 画像のバイト列を新規依存なしで E2E に組み込むのは困難なため、
  JPEG パスはコード上の静的確認に留め、既存 E2E（PNG の MIME 保持）で
  回帰を防ぐ。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| JPEG に quality=0.85（採用） | 画質とファイルサイズの定番のバランスで再現性がある |
| ブラウザ既定に任せる | ブラウザ間で結果が異なる |
| 全形式に quality を指定 | PNG / WebP では意味が異なるため不適切 |

## 影響

- 縮小した JPEG の品質がブラウザに依存せず 0.85 になる。
- その他の形式・機能に影響は無い。
- 追加依存は無い。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 199 項目が
  5 回連続で成功（PNG の MIME 保持を含む）。
- `cargo test --all-targets --locked` は 125 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 解決済み

- JPEG 品質の設定化: ADR-0073 で `jpegQuality`（既定 85、範囲 50〜100）として実装。

## 未決定

- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
