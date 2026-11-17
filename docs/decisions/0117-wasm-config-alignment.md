---
type: Decision
title: "ADR-0117: WasmConfig の AI 既定値整合"
description: "WasmConfig の ai_enabled 既定値を false にし、native の AiConfig と揃える決定を記録する。"
tags:
  - ADR
  - wasm
  - ai
generated:
  by: dsh
  at: "2026-11-18T06:40:00+09:00"
status: accepted
---

## 背景

`WasmConfig::default()` は `ai_enabled: true` だったが、native の
`AiConfig::default()` は ADR-0099 で `enabled: false` に変更済み。
Wasm の AI 実装はスタブで送信はないが、既定値の不整合は将来の
fetch クライアント実装時に「既定で送信する」事故につながる
（ADR-0099 / 0105 の未決定）。

## 決定

- `WasmConfig::default().ai_enabled` を `false` に変更する。
- constructor / getter / setter（`new()` / `ai_enabled()` /
  `set_ai_enabled()`）は維持する（ADR-0042 の `Self::default()` 委譲も維持）。
- JS API シグネチャは不変で、既定値のみが変わる。
  `new WasmConfig().ai_enabled` は `false`、明示 opt-in は
  `config.ai_enabled = true`。
- 他の既定値（`lint_enabled` / `auto_fix`）は変更しない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 既定 false に変更（採用） | native と方針を揃え、将来の送信事故を防ぐ |
| 現状維持 | ADR-0099 の「既定無効」と矛盾する |
| Wasm AI スタブを削除 | JS API 互換性を壊す |

## 影響

- Web リーダーの動作・E2E は不変（wasm-config-auditor が
  `web/src/lib.rs` / E2E に `WasmConfig` 参照が無いことを確認）。
- 配布済み `web/dist` は再ビルドまで旧既定値を返す可能性がある
  （`scripts/verify.sh` は一時 DIST へビルドするため自動更新されない）。
- ADR-0099 / 0105 の当該未決定を解消。

## 検証

- `#[cfg(all(test, target_arch = "wasm32"))]` に 3 テストを追加
  （既定 false、setter、他既定値の不変）。
- `cargo fmt --all` / wasm32 check / wasm32 clippy（`-D warnings`）成功。
- `./scripts/verify.sh` を実行し、6 ゲート・trunk build・WASM パス検査・
  E2E 228 項目がすべて成功した。
- 全単体テスト native core 165 + desktop 28 + web 2。
- **注意**: 追加した 3 テストは wasm32 限定で、通常の
  `cargo test --all-targets`（native）では実行されない。
  implementer は一時クレートで wasm32 の `cargo check --tests` 成功を確認済み。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定（wasm-config-auditor の指摘）

- wasm32 テストの実行手段（`wasm-bindgen-test` 等）がリポジトリに無く、
  `scripts/verify.sh` の wasm32 ゲートは `cargo check` のみ。
- 将来 fetch クライアント実装時に `config.ai_enabled` をどこで読み、
  false 時にどう fail するか。
- `config: JsValue` を `WasmConfig` としてデシリアライズする際の検証。
- ブラウザ版の送信前同意（ADR-0104 は Tauri ネイティブダイアログ前提）。
- wasm の `window.fetch` に対する送信先許容リスト（ADR-0101 は Rust HTTP 前提）。
- Web UI の AI ボタン／設定チェックボックスの配線（ADR-0116 から継続）。
