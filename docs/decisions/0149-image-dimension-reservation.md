---
type: Decision
title: "ADR-0149: 画像自然寸法の属性予約"
description: "EPUB 画像の自然寸法を core で取得し、web で width/height 属性として付与する実装を記録する。"
tags:
  - ADR
  - epub
  - web
generated:
  by: dsh
  at: "2026-11-18T19:00:00+09:00"
status: accepted
---

## 背景

ADR-0136 / 0137 で方針と依存の互換性が決定されていたが、未実装だった。
位置復元 E2E のベースライン（ADR-0140 / 0146）と画像縮小の安定化（ADR-0148）が
完了したため、CLS 対策として寸法予約を実装する。

## 決定

- `imagesize 0.15.0` を core に追加
  （`default-features = false`、`features = ["png","jpeg","gif","webp"]`）。
  実行時依存 0、crates.io / LICENSE で MIT と確認済み（ADR-0137）。
- core に純粋関数 `image_dimensions(bytes) -> Option<(u32,u32)>` を追加。
  - `imagesize::blob_size()` の結果を `u32` へ変換。
  - 0 / 上限 16384px 超 / オーバーフロー / 解析失敗は `None`（fail-closed）。
  - SVG は core では非対応、`None` を返す。
- wasm 公開 API `image_dimensions_wasm(bytes) -> Option<Vec<u32>>` を追加。
- `web/src/lib.rs` の `resolve_images()` で:
  - ラスター画像は `chapter_resource()` のバイト列から自然寸法を取得し、
    `width` / `height` を設定（両方必須）。
  - SVG はルート要素の既存 `width` / `height` を尊重して付与。
    viewBox のみ / 0 / 上限超 / 非数は付与しない。
  - `src` を削除した画像は `width` / `height` も削除（ゴーストボックス対策）。
- 属性は**縦横比の予約専用**とし、巨大画像警告・縮小判定は
  従来どおり `naturalWidth` / `naturalHeight` に依存する。
- `THIRD_PARTY_NOTICES.md` に imagesize の MIT 通知（Copyright (c) 2017 Maiddog、
  crates.io URL、MIT 全文）を追記。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| core 取得＋web 注入（採用） | ADR-0136 の方針を実装。wasm/native 共通 |
| web で DOM デコード後に計測 | CLS 対策にならない |
| self-made ヘッダーパーサ | ADR-0137 で却下済み |

## 影響

- E2E を 2 件追加（`Check(check=214 + equal=17 = 231)`）。
  - `#pic` に自然寸法と一致する 1x1 の width/height が付く。
  - `#evil-svg` の既存 20x20 属性と Blob 内属性が維持される。
- `Cargo.lock` は imagesize のエントリと core 依存の 1 行追加のみ
  （不要な syn 変更は元に戻した）。
- 新規依存は MIT の imagesize のみ、unsafe なし。

## 検証（Runtime Proof）

- `cargo fmt` / `check --all-targets --locked` / `test --all-targets --locked` /
  `clippy --all-targets --locked -- -D warnings`: すべて成功。
- wasm32 core / web check、wasm-bindgen-test: 成功。
- `cargo audit --no-fetch`: 脆弱性 0、警告 baseline 10 のまま。
- `python3 -m py_compile tools/verify-web-reader.py`: 成功。
- `./scripts/verify.sh` rc=0、E2E **231 項目 5 回連続 0 失敗**。
- 全テスト: core 192 + desktop 56 + web 2、wasm-test 4。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定（dimension-auditor の指摘）

- ~~作者指定 `width` / `height` の扱い~~ ADR-0150 でサニタイザによる数値検証を追加済み（上書き自体は継続）。
- 縮小後も元寸法の属性が残ることの微シフト影響（ADR-0136 / 0148 から継続）。
- 上限 16384px の根拠の最終化、CLS 直接計測 E2E、
  `.reader-cover` の対象範囲、`IMAGE_SETTLE_MS = 2000` の根拠。
- imagesize がヘッダーのみ読むため、途中破損画像でも寸法を返し得る
  （完全デコード検証は要件外としたが、推測を含む）。
- 法務レビュー（MIT 通知の十分性）と CI 復帰後の audit パス確認。
