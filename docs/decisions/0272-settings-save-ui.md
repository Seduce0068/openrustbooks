---
type: Decision
title: "ADR-0272: Tauri 設定保存 UI の接続"
description: "完全な ConfigView DTO 往復と Tauri mock E2E を用いて、desktop のみ設定保存 UI を接続する決定を記録する。"
tags:
  - ADR
  - tauri
  - settings
generated:
  by: dsh
  at: "2026-11-20T06:08:00+09:00"
status: accepted
---

## 背景

ADR-0271 は web 側 `SettingsView` が `ConfigPatch` へ Serialize できず、
allow-list View の差分 23 項目と `expected` が不足していることを記録した。
IPC 経路（ADR-0270）と UI 起点 CAS / 構造化エラー（ADR-0269）は完了済みで、
段階設計の残りは DTO 往復、a11y、E2E 成功経路、実 WebView 検証だった。

## 決定

- web 側 `Settings*View` を desktop `ConfigView` と同形に拡張し、
  `Serialize` + `Deserialize` を derive する。
- `revision` を保存時に `ConfigPatch.expected` へコピーし、
  `revision` キーは IPC へ送らない。
- secret / path / `extra_headers` / API Key 値は web DTO に追加しない
  （`has_*` フラグのみを往復）。
- `save_config()` は `update_config_patch` を呼び、
  `CommandError.code` のみを日本語固定 status へ変換する。
  生 message / path / 秘密情報は表示しない。
- `fetch_config()` 成功時のみ editable とし、
  browser は readonly / disabled / 既存説明 /
  「デスクトップ版のみ利用可能」を維持する。
- a11y は `label[for]` + `id`、`aria-invalid`、`aria-describedby`、
  `role="status" aria-live="polite"` を満たす。
- E2E に Tauri mock 成功経路を 1 件追加し、
  期待総数を **255 → 256**（check 239 + equal 17）とする。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 完全 DTO 往復 + desktop のみ有効化（採用） | allow-list 契約と browser fail-closed を維持できる |
| 既存 SettingsView に expected だけ追加 | desktop デシリアライズ失敗／隠しフィールド上書きのリスク |
| Web DTO に secret / path を追加 | ADR-0102 / 0103 / 0186 の redaction 契約違反 |

## 影響

- desktop で設定変更と保存が可能になる（実 WebView は未検証）。
- browser E2E の既存 2 件（fallback / readonly）は不変。
- E2E check 名・件数は 256 に更新し、
  `check-e2e-count-consistency.sh` / traceability / 規約文書を同期した。
- API Key 値の編集は引き続き不可（`has_api_key` のみ表示）。

## 検証（Runtime Proof）

- `cargo test --all-targets --locked`: core **287** / desktop 82 / web 2。
- `wasm-pack test --node --locked`: 14 passed。
- Tauri mock E2E: get_config 完全 ConfigView、
  update_config_patch の `expected + 1`、固定成功文言、
  JS エラー 0 を 1 check で検証。
- AST: `checker.check` 239 + `checker.equal` 17 = **256**、
  `Checker(expected=256)`、同名 check 0、文書の check 名 256 件と一致。
- `./scripts/verify.sh`: 全ゲート成功、
  E2E **256 passed / 0 failed**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 実 Tauri WebView での往路 / 復路 / 表示 / a11y 検証
  （mock は代替ではない、ADR-0253 実装前提 6）。
- API Key 値の編集可否（専用 command または config.toml 直接、ADR-0134 / 0135）。
- 外部 JS 利用者の有無（`withGlobalTauri` 有効化の破壊的影響）。
- CSP nonce/hash 化（`'unsafe-inline'` 残存、ADR-0091 / 0264 / 0268）。
- CAS 衝突時の UX（自動再取得 vs 手動再読込、ADR-0269 / 0271）。
