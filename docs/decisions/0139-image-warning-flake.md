---
type: Decision
title: "ADR-0139: 画像警告フレークの修正"
description: "位置適用の 2 秒 deadline を過ぎて読み込まれる巨大画像を load イベントで検出し、警告の取りこぼしを防ぐ決定を記録する。"
tags:
  - ADR
  - web
  - testing
generated:
  by: dsh
  at: "2026-11-18T13:00:00+09:00"
status: accepted
---

## 背景

ADR-0138 の 5 回連続実行で `5000px の画像で警告が出る` が 1 回失敗
（`status=''`）。一過性表示が疑われたが、コードに基づく切り分けの結果、
**「出た後に消えた」のではなく「最初から出ていなかった」**ことが判明した。

- `status` を消すのは `render_chapter` の `set_status.set(String::new())` のみ
  （web/src/lib.rs:1893 付近）で、タイマークリアは無い。
- 巨大画像警告は `apply_position_frame` の最終コールバックで
  `find_large_image()` を 1 回呼ぶだけ（:754-755）。
- そのコールバックは `IMAGE_SETTLE_MS = 2000` の deadline に達すると
  `images_settled = false` のまま強制的に進む（:690）。
  5000px 画像の自然サイズ確定が 2 秒を超えると警告が出ない。

## 決定

- `Reader` コンポーネントで `document` の `load` イベントを capture 監視し、
  画像ロード完了時に `find_large_image()` を再評価して警告を `set_status` に設定する。
  - `load` はバブルしないため `add_event_listener_with_callback_and_bool("load", ..., true)`。
  - リスナーはページ寿命まで有効とし `forget()` で保持（変更 20 行）。
- 既存の警告閾値（4000px **超**）、警告クリア、remote-resources 警告との
  優先関係は変更しない。
- `tools/verify-web-reader.py` は変更せず、期待総数 229 を維持。
- 単純な警告の永続化は、4000px 境界や remote-resources の否定チェックを
  壊すため採用しない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| load イベントで再検出（採用） | deadline に依存せず、警告の意味も保てる |
| 警告を永続化 | 否定チェック・他警告との優先順位が壊れる |
| MutationObserver 監視 | 「出ていない」ケースには効かない |
| ポーリング短縮 | 主因が「出ない」ため無効 |

## 影響

- 2 秒を超えて読み込まれる巨大画像でも警告が出る。
- イベントリスナーはページ寿命で 1 つだけ（forget による意図的な保持）。
- 新規依存なし、unsafe なし。

## 検証

- 修正後ビルドで E2E **5 回連続 229/0**（4000px 境界・remote-resources
  の否定チェックも全回成功）。
- `./scripts/verify.sh` rc=0（fmt / check / test / clippy / wasm × 2 /
  wasm-test / audit / trunk build / wasm path check / E2E）。
- 全単体テスト core 188 + desktop 56 + web 2、wasm-test 3、
  audit 脆弱性 0 件。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- ~~位置復元系の 10 回連続実行は未実施~~ ADR-0140 で実施済み。20 回は未実施。
- `IMAGE_SETTLE_MS = 2000` 自体の根拠と再調整（ADR-0113 から継続）。
- 属性付与後の 50px / 30px 許容値の再評価（ADR-0136 / 0138 から継続）。
