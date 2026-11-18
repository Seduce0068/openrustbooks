---
type: Decision
title: "ADR-0192: Reader 終了時の Blob URL 解放"
description: "Reader アンマウント時に残余の object URL を revoke し、SPA 往復によるメモリ残存を防ぐ決定を記録する。"
tags:
  - ADR
  - web
  - memory
generated:
  by: dsh
  at: "2026-11-19T07:10:00+09:00"
status: accepted
---

## 背景

`object_urls` / `cover_url_state` は章再描画時とファイル切替時に
`revoke_object_url()` されていたが、Reader がアンマウントされる
`on_cleanup` では解放していなかった（R6 TD-M09）。
SPA でルートを往復するたびに画像 Blob がメモリに残る。

## 決定

- Reader の `on_cleanup` で次を実行する:
  - `object_urls.borrow_mut().drain(..)` の全 URL を `revoke_object_url()`
  - `cover_url_state.borrow_mut().take()` の URL を `revoke_object_url()`
- `on_cleanup` は `FnOnce() + Send + Sync + 'static` を要求するため、
  `Rc<RefCell<...>>` を `SendWrapper` で包んでキャプチャする
  （ADR-0166 の既存パターンを再利用）。
- 章再描画・ファイル切替時の既存 revoke は維持する。
- E2E は 249 件を維持（直接検証は fragile のため見送り、後述）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| on_cleanup で残余を drain（採用） | アンマウント時の取りこぼしを確実に解放 |
| 既存 revoke のみ | SPA 往復で Blob が残存する |
| E2E で直接検証 | Page.navigate は full reload で Owner 破棄を捕捉できない |

## 影響

- cover URL は `cover_url_state` のみに格納され `object_urls` とは重複しない。
- ダウンスケールで生成した URL は `object_urls` に登録済みのため drain 対象。
- 二重 revoke は `URL.revokeObjectURL()` の no-op 仕様と
  既存の `let _ = ...` により実害なし。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- `cargo fmt --all -- --check`、
  `cargo check -p openrustbooks-web --target wasm32-unknown-unknown --locked`、
  `./scripts/verify.sh`: すべて成功。
- E2E **249 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `canvas.to_blob` コールバックが cleanup 後に完了すると
  URL が `object_urls` へ再登録され、unmount 後は回収されないレース
  （既定オフ機能・発生ウィンドウ小、推測）。
- このレースを防ぐための世代ガード導入
  （GOAL の「ダウンスケール挙動を変更しない」制約で本次見送り）。
- cleanup 発火の直接 E2E 検証（ADR-0166 の未決定を継続）。
- `send_wrapper` の re-export パス確認（ADR-0166 から継続）。
