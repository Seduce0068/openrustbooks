---
type: Decision
title: "ADR-0225: 壊れた画像のセトリング待ち解消"
description: "画像の error を記録して失敗画像を待ち対象外にし、章ごとの 2 秒待ちを防ぐ決定を記録する。"
tags:
  - ADR
  - web
  - performance
generated:
  by: dsh
  at: "2026-11-19T15:47:00+09:00"
status: accepted
---

## 背景

`images_settled()` は `src` を持つ全画像に
`complete() && natural_width() > 0` を要求していた（R6 TD-M10）。
読み込み失敗画像は `complete()` が true のまま自然幅が 0 なので、
`IMAGE_SETTLE_MS = 2000` の deadline まで位置復元が待たされた。
章を切り替えるたびに同じ待ちが発生していた。

## 決定

- `resolve_images()` が Blob URL を設定した `<img>` に
  `data-render-generation` / `data-render-chapter` を付与する（`src` は削除しない）。
- `ReaderPage` で document capture の `error` リスナーを 1 回登録し、
  生成時の generation / chapter が現在値と一致する画像にだけ
  `data-image-error = "1"` を書き込む。
  旧章や Blob URL revoke 由来の古い error は無視し、
  cleanup でリスナーを解除する。
- 完了条件を
  `complete() && (natural_width() > 0 || data-image-error == "1")`
  に拡張する。
- 正常画像の自然幅待ち、`src` なし画像の除外、
  `expected_images` / 2000ms deadline / generation ガード /
  再試行ループは維持する（ADR-0113 / 0114 / 0136 / 0159）。
- E2E を 1 件追加し、期待総数 253 → **254**（check 237 + equal 17）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| error 属性で失敗を記録（採用） | 正常画像の待ちを維持しつつ失敗だけ特別扱いできる |
| `complete()` のみへ緩和 | ADR-0113 の未確定画像バグが復活する |
| error 時に `src` を削除 | ADR-0114 の expected 不一致で逆に 2 秒待ちになる |

## 影響

- 壊れた画像を含む章の位置復元が 2 秒未満で完了する。
- 正常画像は従来どおり自然サイズ確定まで待つ（CLS 契約維持）。
- Blob URL の寿命管理・画像サニタイズ・generation ガードは変更しない。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- E2E「壊れた画像を含む章の位置復元が 1.5 秒未満で完了する」が ok。
  - 不正バイトの `broken.png` を含む章を生成。
  - `#broken` が `blob:` src・`complete() === true`・`naturalWidth === 0` を確認。
  - 本文出現から位置適用完了までの `performance.now()` 差が 1500ms 未満。
- AST 抽出: check 237 + equal 17 = **254**、同名重複 0。
- `python3 -m py_compile`、wasm-web check、`./scripts/verify.sh`: 成功。
- E2E **254 項目すべて成功**。
- `docs/e2e-traceability.md` を 254 件へ追従。新規 check を VC-004 の読書位置行へ
  割り当て、AST 照合で missing / extra 0（ADR-0206 の運用課題を解消）。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- error 発火が deadline 前か後かで効果が変わる timing 依存。
- Firefox / Safari で `complete()` / `naturalWidth` / error 挙動が異なる可能性
  （実機 5 ブラウザ未検証、ADR-0159 継続）。
- 章切替時の Blob URL revoke と error の競合（現状は generation で排他）。
