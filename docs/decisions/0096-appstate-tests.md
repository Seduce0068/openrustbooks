---
type: Decision
title: "ADR-0096: Desktop AppState の状態遷移テスト"
description: "desktop の AppState に Tauri ランタイム不要の状態遷移テストを追加する決定を記録する。"
tags:
  - ADR
  - desktop
  - testing
generated:
  by: dsh
  at: "2026-11-18T01:48:00+09:00"
status: accepted
---

## 背景

`desktop/src/state/mod.rs` にテストが無く、`is_unsaved` /
`has_unsaved_changes` は未保存クローズ拒否を支配する重要な状態だった
（test-scanner #5）。Tauri ランタイムなしで検証可能。

## 決定

- `#[cfg(test)] mod tests` を追加し、次を検証する。
  - 生成直後は未保存変更なし。
  - `add_document` → `mark_unsaved` で `is_unsaved` / `has_unsaved_changes` が true。
  - `mark_saved` で false に戻る。
  - `remove_document` で該当 id が消え、未保存変更が false になる。
  - `config()` → 変更 → `update_config()` の往復。
- 新規依存は追加しない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| Tauri なしの単体テスト（採用） | 高速で決定論的、CI でも動く |
| Tauri ランタイムで E2E | 重く、状態遷移の単体検証には不向き |
| テストなし | 未保存状態の回帰を検出できない |

## 影響

- Core 126 件に加え desktop 2 件のテストが増え、合計 128 件になった。
- `desktop/src/main.rs` や Tauri コマンドは変更しない。

## 検証

- `cargo test --all-targets --locked`: Core 126 件 + desktop 2 件、すべて成功。
- 途中で `RwLock<AppConfig>` への直接アクセスと既定値（AI は既定で有効）
  を誤って前提として失敗したが、`config()` 経由と既定値に合わせて修正した。
- `./scripts/verify.sh` を実行し、6 ゲート・trunk build・WASM パス検査・
  E2E 228 項目がすべて成功した。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `save_config()` のディスク書き込みテスト（一時ファイルが必要）。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
EOF
