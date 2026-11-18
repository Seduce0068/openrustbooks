---
type: Decision
title: "ADR-0137: imagesize の wasm32 互換性検証"
description: "画像寸法取得クレート imagesize が wasm32 で使えることを隔離検証で確認し、段階導入の前提を解消する。"
tags:
  - ADR
  - wasm
  - dependency
  - epub
generated:
  by: dsh
  at: "2026-11-18T11:55:00+09:00"
status: accepted
---

## 背景

ADR-0136 は PNG / JPEG / WebP / GIF の寸法取得に `imagesize 0.15.0`
（MIT・軽量）を第一候補としたが、wasm32-unknown-unknown でのビルド可否は
未確認だった（ADR-0136:85）。

## 隔離検証の結果（imagesize-wasm-checker）

- 実バージョン: **imagesize v0.15.0**（crates.io から解決）。
- `rustc` / `cargo` 1.100.0、`wasm32-unknown-unknown` ターゲット導入済み。
- `cargo check --target wasm32-unknown-unknown` は **exit code 0**。
- `imagesize::blob_size(bytes).ok().map(|d| (d.width, d.height))` が
  `Option<(usize, usize)>` として型チェック成功（故意の型エラーでは
  exit 101 になることを確認し、型検査が働いていることを検証）。
- 推移依存なし（`cargo tree --target wasm32-unknown-unknown` は imagesize 単独）。
- feature 絞り込み版（`default-features = false` + `png,jpeg,gif,webp`）と
  既定版の両方で wasm check 成功。差はコード量のみ
  （release rlib: 絞り込み 204,168 bytes / 既定 675,542 bytes、
  メタデータ含むため厳密な wasm 増分ではない）。
- 一時ディレクトリ（`/tmp/orb-imagesize-check*`）は削除済み。
  リポジトリの Cargo.toml / Cargo.lock は未変更。

## 監査で補った調査（image-dimension-auditor）

| 観点 | 結果 |
| --- | --- |
| ライセンス | MIT / Copyright (c) 2017 Maiddog（crates.io と LICENSE 原文で確認）。AGPL-3.0-or-later と一方向互換と解釈するが **推測・法務未確認** |
| 脆弱性 | RUSTSEC advisory DB のコード検索で imagesize 言及 0 件（網羅性は保証しない）。ローカル `cargo audit` は未実施 |
| サイズ | crate tarball 29,653 bytes / 1,732 行（外部事実）。wasm 成果物への増分は未測 |
| 保守性 | 2026-07-09 公開、直近 DL 22,220,909（外部事実） |
| 実行時依存 | 0（dev 依存のみ） |

## 決定

- `imagesize 0.15.0` は wasm32 で利用可能であり、ADR-0136 の
  「自然寸法取得の第一候補」を **採用可能**と確認する。
- ただし ADR-0136 の判断（今回は実装しない）は維持する。
  実装は別 ADR で、E2E 回帰テストの整備を先に行う。
- 実装時の条件:
  - `default-features = false, features = ["png","jpeg","gif","webp"]` を採用
    （互換性に必須ではないが、コード量削減に有効）。
  - `THIRD_PARTY_NOTICES.md` に imagesize の MIT 通知を追記
    （ AGPL 運用上の条件、**推測**）。
  - 依存追加後に `cargo audit --no-fetch` を再実行し、脆弱性 0・
    警告 baseline 10 のままであることを確認する。
- 自作ヘッダーパーサへは落とさない（ADR-0136:49 の方針を維持）。

## 影響

- コード変更なし。既存の 246 テスト＋wasm 3、E2E 229 項目は維持。
- ADR-0136 の未決定「wasm32 ビルド可否」を解消。

## 検証

- 隔離クレートでの wasm32 check と `blob_size` 型チェックが成功。
- 一時ファイルは削除済み、リポジトリは未変更。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定（ADR-0136 / 監査から継続）

- 作者指定 `width` / `height` の扱い（信頼／上書き／クランプ）。
- 0 / 非数 / 単位付き / 上限超の拒否基準（実装側で明示）。
- 縮小後の属性更新の有無、viewBox のみ SVG の扱い。
- CLS 直接計測 E2E（`PerformanceObserver('layout-shift')`）の導入。
- ~~位置復元 E2E のベースライン整備~~ ADR-0140 / 0146 で実施済み。
- `.reader-cover` を範囲に含めるか、`IMAGE_SETTLE_MS = 2000` の根拠。
- 5 ブラウザ実機検証（現状は Chrome 157 のみ）。
- 法務レビュー（MIT 通知の十分性）と CI 復帰後の audit パス確認。
