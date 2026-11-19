---
type: Decision
title: "ADR-0205: WasmConfig の削除"
description: "未使用の WasmConfig 公開 API を削除し、設定モデル重複の 1 つを解消する決定を記録する。"
tags:
  - ADR
  - wasm
  - config
generated:
  by: dsh
  at: "2026-11-19T10:05:00+09:00"
status: accepted
---

## 背景

`core/src/wasm.rs` の `WasmConfig` は wasm32 向け公開型だったが、
`web/src/lib.rs` / desktop からは未参照で、
wasm unit test 3 件のみが依存していた（R6 PR-M08 / TD-M08 の一部）。
`auto_fix` の権威は `LintConfig`（ADR-0160 / 0194）、
AI 既定値は native `AiConfig`（ADR-0117 / 0099）にあり、
設定モデル 4 重化の要因だった。

## 決定

- `WasmConfig` の struct / `Default` impl / `#[wasm_bindgen]` impl
  （constructor + 全 getter / setter）を削除する。
- wasm test モジュールの import と、`ai_disabled_by_default` /
  `set_ai_enabled_switches_state` / `other_defaults_are_unchanged` の
  3 test を削除する。
- `auto_fix` 権威コメントを `LintConfig`（ADR-0160 / 0194）へ更新する。
- 他の wasm API（`WasmMarkdownLinter` / `render_markdown_wasm` /
  `WasmEpubBook` / `image_dimensions_wasm` 等）は維持する。
- 歴史 ADR（0042 / 0105 / 0117 / 0156 / 0160 / 0179 / 0194）の本文は変更せず、
  本 ADR で supersede を明記する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 削除（採用） | 本番参照 0、権威が別にあるため重複を減らせる |
| 維持 | 未使用 API と誤った権威の誤解が残る |
| 非公開化のみ | wasm-bindgen 型を残す意味が薄い |

## 影響

- wasm test は **10 → 7 件**（WasmConfig 依存 3 件を削除）。
- Rust ソースの `WasmConfig` 参照は 0 件。
- wasm-bindgen の公開型を削除するため、
  リポジトリ外 JS が `new WasmConfig()` を使っていた場合は破壊的変更になり得る
  （リポジトリ内にその証拠はなし）。
- 残る設定モデルは `AppConfig` / `ReaderSettings` / `SettingsView` の 3 つ
  （別タスク）。
- 新規依存なし、unsafe なし、E2E 251 維持。

## 検証（Runtime Proof）

- `grep -rn 'WasmConfig' --include='*.rs' core/src web/src desktop/src` → 0 件。
- `cargo test --all-targets --locked`: core 255 + desktop 72 + web 2。
- `wasm-pack test --node --locked`: **7 passed**（旧 10 から削除分 3 件）。
- `cargo clippy --all-targets --locked -- -D warnings`、
  wasm32 core / web check、`./scripts/verify.sh`: すべて成功。
- E2E **251 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 旧 ADR の `status` を `superseded` に更新するか（現状は accepted のまま、
  本 ADR で上書き）。
- 外部利用者への deprecation / 移行手順（リポジトリ外利用は未確認）。
- 残る 3 重（AppConfig / ReaderSettings / SettingsView）の統合。
- ADR-0179:73 の `auto_save_delay` 供給元の再定義（WasmConfig は消滅）。
- ADR-0160 / 0117 の未決定の解消表明。
