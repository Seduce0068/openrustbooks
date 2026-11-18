---
type: Decision
title: "ADR-0148: 画像縮小の完了シグナルとリトライ"
description: "画像縮小の完了を data-downscaled で明示し、未確定画像のリトライと MIME 属性化で E2E フレークを解消する決定を記録する。"
tags:
  - ADR
  - web
  - testing
generated:
  by: dsh
  at: "2026-11-18T18:35:00+09:00"
status: accepted
---

## 背景

ADR-0141 で待機改善を入れた後も画像縮小 E2E がフレークした
（10 回中 1〜2 回、`naturalWidth=4500` のまま）。
診断では `src` が blob URL、`status` に巨大画像警告が出ており、
完了シグナルが無いため「スキップ」と「遅延」を区別できなかった。

## 決定

- `resolve_images()` で `data-source-mime` 属性に元画像の MIME を保持する。
- `downscale_large_images()` を次のように変更:
  - `fetch_blob_type()`（blob URL への fetch）を廃止し、
    `data-source-mime` から MIME を取得（無効／未知なら PNG 既定）。
  - 自然サイズ未確定（`naturalWidth == 0 || naturalHeight == 0`）の
    4000px 超候補は今回スキップし、100ms × 最大 20 回のリトライを
    1 呼び出しにつき高々 1 つだけスケジュールする。
  - 縮小後の Blob URL を `src` へ設定する前に `load` リスナーを登録し、
    完了後に `data-downscaled="1"` を付与する（`set_downscaled_src()`）。
  - `data-downscaled="1"` 済み画像は再処理しない。
- E2E は完了待ちを
  `#wide[data-downscaled="1"] && naturalWidth <= 2000`（AND）に変更。
  チェック名・期待総数 229・閾値は不変（待ち条件のみ）。
- 縮小ポリシー（4000px 超、`max(2000, content_width)`、MIME 保持、
  object_urls / revoke）は変更しない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 完了シグナル＋リトライ＋MIME 属性（採用） | 原因を根本から解消し、E2E も決定的に待てる |
| さらに待機時間を延長 | 「永遠に来ない」ケースを隠すだけ |
| 属性単独で完了とみなす | naturalWidth と乖離する可能性（監査指摘） |

## 影響

- 画像 src に `data-source-mime`、縮小後に `data-downscaled` が付与される。
- `fetch_blob_type()` を削除し、blob fetch に伴うハング要因を除去。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- `cargo fmt --all -- --check` / wasm32 check / clippy: すべて exit 0。
- `python3 -m py_compile tools/verify-web-reader.py`: 成功。
- `./scripts/verify.sh` rc=0（E2E 229/0）。
- 修正後 E2E **10 回連続 229/0**（画像縮小フレーク再発なし）。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- generation guard を downscale callback / load リスナーまで拡張するか
  （古い章の callback が object_urls を汚す可能性、推測）。
- `applying=false` を縮小完了まで遅らせるか（ADR-0141 から継続）。
- `content_width > 2000` のときの「2000px 以下」文言と実装のずれ（ADR-0067 から継続）。
- 縦書き時の画像縮小基準の論理軸化（ADR-0145 から継続）。
- 10 回連続の統計的限界、CLS 計測、5 ブラウザ実機検証。
