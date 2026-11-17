---
type: Decision
title: "ADR-0113: 画像完了判定の強化"
description: "images_settled() に自然幅の確認を追加し、complete() だけでは見逃す画像を待つ決定を記録する。"
tags:
  - ADR
  - web
  - bugfix
generated:
  by: dsh
  at: "2026-11-18T05:35:00+09:00"
status: accepted
---

## 背景

MDN によれば `HTMLImageElement.complete` は `src` 未設定／空／broken／
画像読み込み無効でも `true` を返す。現状の `images_settled()` は
`complete()` のみで判定しており、自然サイズが確定していない画像を
「完了」とみなす可能性があった。位置復元は画像の自然サイズに依存する
（ADR-0020 / 0037 / 0046）。

## 決定

- `images_settled()` の完了条件を `complete() && natural_width() > 0` に変更。
- `with_source >= expected`、`src` なし画像を待たない挙動、
  `IMAGE_SETTLE_MS = 2000` の fail-open deadline は維持。
- 位置復元・スクロール補正のロジック、`loading="lazy"` 不採用
  （ADR-0034 / 0112）は維持。
- ついでに clippy の既存 baseline 5 件（冗長な再束縛 4 件と
  `html_input.focus()` の unused Result）も解消した。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| `complete()` ＋自然幅（採用） | 自然サイズ未確定画像を待ち、位置復元のズレを防ぐ |
| `complete()` のみ | broken 画像を見逃す |
| `error` イベントで完了扱い | 今回の範囲外（別 ADR で検討） |

## 影響

- 正常な PNG / JPEG / WebP / SVG Blob は即座に条件を満たすため、
  通常の読み込み速度は変わらない。
- 破損画像・未知 MIME は `naturalWidth == 0` のままのため、
  章移動ごとに 2 秒 deadline まで待つ（fail-open で位置適用は進む）。
- サニタイズ済み SVG Blob は `width` / `height` 属性を保持するため
  通常は問題ないが、viewBox のみの SVG はブラウザ依存（推測）。
- E2E の `#tall` / `#wide` / `#pic` / 位置復元チェックは新条件と整合。

## 検証

- `cargo fmt --all` / wasm32 check / wasm32 clippy（`-D warnings`）すべて成功。
- `./scripts/verify.sh` を実行し、6 ゲート・trunk build・WASM パス検査・
  E2E 228 項目がすべて成功した。
- 全単体テスト core 165 + desktop 28。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定（settle-auditor の指摘）

- 破損画像の 2 秒待ちを許容するか、`error` イベント等で即完了扱いにするか。
- `IMAGE_SETTLE_MS = 2000` の根拠と遅い端末での再調整要否。
- viewBox のみ SVG の自然幅（ブラウザ依存、推測）。
- `expected_images` に src 削除画像が残る既存負債（ADR-0112 の段階導入で
  「可視／位置に影響する画像」定義と統合する）。
