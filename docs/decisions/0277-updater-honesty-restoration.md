---
type: Decision
title: "ADR-0277: updater 死設定の fail-closed 復元"
description: "設定保存 UI の editable 化で失われた ADR-0222 の固定表示と、check_updates の保存時 fail-closed を復元する決定。"
tags:
  - ADR
  - updater
  - settings
  - security
generated:
  by: dsh
  at: "2026-11-20T07:27:00+09:00"
status: accepted
---

## 背景

ADR-0222 は updater 未配線を設計受容し、
`check_updates` の値に関わらず「更新確認: 未配線」を表示すると決定した。
しかし ADR-0272 の設定保存 UI editable 化で `check_updates` が通常の checkbox になり、
DTO 往復で config.toml へ保存されるようになった。
updater は plugin 初期化のみで、
`UpdaterExt::check` / pubkey / endpoints は存在しない。

## 決定

- web 側設定画面から `check_updates` の editable checkbox を削除し、
  「更新確認: 未配線（updater は実装されていません）」を常時表示する。
- `config.general.check_updates` は DTO 往復のため残す。
- `merge_secrets()` で `merged.general.check_updates = current.general.check_updates`
  とし、JS からの変更を config.toml へ保存しない（二段構えの fail-closed）。
- `scripts/check-updater-honesty.sh` を追加し、次を静的検査する:
  - `UpdaterExt::check` / `.check_update` / `.check()` が無い（検出パターンの拡張は ADR-0279）
  - `plugins.updater.pubkey` / `endpoints` が無い
  - 固定表示が残り `id="settings-check-updates"` が無い
  - 現在値復元行と単体 test が残る
- `verify.sh` と `ci.yml` の `extra_gates` から同じ script を呼ぶ。
- E2E は check 名・件数（256）を不変とし、
  既存 browser readonly check と Tauri mock check の条件のみを拡張する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| UI 固定表示 + 保存時無視 + 静的ゲート（採用） | ADR-0222 の契約を復元しつつ再発を防止できる |
| UI のみ修正 | DTO 経由で死設定が保存され続ける |
| UpdaterExt::check を配線 | pubkey / endpoint / 署名が無く fail-closed で動作しない |
| check_updates を DTO から削除 | ADR-0272 の完全 DTO 往復契約を壊す |

## 影響

- 利用者は更新確認を変更できず、常に未配線と表示される。
- JS から `check_updates` を送っても現在値が保持され、config.toml は更新されない。
- 公開鍵 / endpoint / 自動更新 / 署名 secrets は導入しない。
- `createUpdaterArtifacts: true` は成果物生成フラグの残置として維持。

## 検証（Runtime Proof）

- 負向 test **8/8 PASS**: baseline rc=0、
  pubkey / endpoints / UpdaterExt::check / 固定文言欠落 /
  input 復活 / merge 行欠落 / test 欠落はすべて rc=1。
- `cargo test --all-targets --locked`: core 287 / desktop **83** / web 2。
- `wasm-pack test --node --locked`: 14 passed。
- AST: check 239 + equal 17 = 256、`Checker(expected=256)`。
- `./scripts/verify.sh`: 新ゲート rc=0、全ゲート成功、
  E2E **256 passed / 0 failed**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- ADR-0282 で updater plugin 登録そのものを削除済み。`check-updater-honesty.sh` は登録なしを要求する。

- ADR-0222 / 0271 / 0272 / runbook の優先順位と、
  「DTO 不変・E2E 253 維持」という履歴条項の扱い。
- 実 Tauri WebView で固定表示が描画されるか（ADR-0273 の環境ブロッカー）。
- `check_updates` を DTO から完全に削除するか（設定形式の互換判断）。
- 既存 E2E check の条件拡張を「check 名・件数不変」の範囲とみなす規約化。
- `createUpdaterArtifacts: true` 残置の再確認。
