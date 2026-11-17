---
type: Decision
title: "ADR-0071: 縮小画像の MIME 保持"
description: "画像縮小時に元画像の MIME タイプを保持する決定を記録する。"
tags:
  - ADR
  - epub
  - web
  - performance
generated:
  by: dsh
  at: "2026-11-17T10:15:00+09:00"
status: accepted
---

## 背景

ADR-0067 の未決定「縮小後の画質比較と、JPEG 再エンコードの選択」。
`canvas.to_blob()` の既定（PNG）で再エンコードしていたため、JPEG の
写真画像でも PNG になり、ファイルサイズが増えることがあった。

## 決定

- 縮小前に元画像の Blob URL を `fetch` して Blob の `type` を取得する。
- MIME が `image/jpeg` / `image/png` / `image/webp` のいずれかなら
  `canvas.to_blob_with_type()` で同じ形式に再エンコードする。
- それ以外（GIF 等）や MIME 取得失敗時は既定（PNG）にフォールバックする（fail-safe）。
- `downscale_large_images` を async 化し、呼び出し側で `spawn_local` する。
- 縮小機能の既定オフ（ADR-0067）や、表示サイズを変えない方針は維持する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 元 MIME を保持（採用） | 写真は JPEG のまま、透過画像は PNG のままになる |
| 常に PNG | JPEG の写真が巨大化する |
| 常に JPEG | 透過情報が失われる |

## 影響

- 縮小後の Blob が元と同じ形式になるため、画質とファイルサイズの
  バランスが改善される。
- GIF のアニメーションは縮小時に失われる（ADR-0067 の既知の制約）。
- 非同期 fetch が 1 回増えるが、縮小はオプトインのため通常は影響しない。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 199 項目が
  5 回連続で成功。
- PNG 画像を縮小した後も Blob の MIME が `image/png` であることを確認した。
- `cargo test --all-targets --locked` は 125 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- JPEG の再エンコード品質（quality）の最適値。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
