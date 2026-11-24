---
type: Decision
title: "ADR-0092: IME 変換中のショートカット抑制"
description: "IME の変換中はキーボードショートカットを発火させない決定を記録する。"
tags:
  - ADR
  - web
  - accessibility
generated:
  by: dsh
  at: "2026-11-18T00:20:00+09:00"
status: accepted
---

## 背景

`key_handler` は `isComposing` を確認していなかったため、日本語入力の変換確定時に
`d` や `[` / `]` などがショートカットとして扱われ、意図せず章移動や
表示切り替えが起こる可能性があった（todo.md B04 / C01 の一部）。

## 決定

- `key_handler` の先頭で `event.is_composing()` が true なら return する。
- E2E で `isComposing: true` のキーイベントを発行し、ダーク表示が
  切り替わらないことを確認する。
- 期待 check 総数を 223 に更新する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| `isComposing` でガード（採用） | 既存のキー処理を変えずに誤操作を防ぐ |
| Compose 中だけ別ハンドラ | 状態管理が増え、効果は同じ |
| 入力欄フォーカスのみで判定 | 本文での IME 変換を防げない |

## 影響

- IME 変換中にショートカットが発火しなくなる。
- 通常のキーボード操作は変わらない。
- 追加依存は無い。

## 検証

- E2E 223 項目を 5 回連続実行し、すべて成功。
- `./scripts/verify.sh` を実行し、6 ゲート・trunk build・WASM パス検査・
  E2E 223 項目がすべて成功した。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `keydown` 以外のイベント（`keyup` 等）での IME 判定。
- `compositionstart` / `compositionend` の明示的な追跡。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
