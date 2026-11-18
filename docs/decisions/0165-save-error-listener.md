---
type: Decision
title: "ADR-0165: save-error リスナーの送信先分離"
description: "保存失敗通知を送信先別イベント名へ分離し、Editor / Reader の status 競合を解消する決定を記録する。"
tags:
  - ADR
  - web
  - testing
generated:
  by: dsh
  at: "2026-11-18T23:20:00+09:00"
status: accepted
---

## 背景

ADR-0154 で `openrustbooks:save-error` と Reader のリスナーを追加し、
ADR-0164 で Editor にも同じイベントのリスナーを追加した。
両画面が同時に存在する場合、片方の保存失敗が他方の status を
上書きし得る（ADR-0164 の未決定）。

## 決定

- `web-sys` の `CustomEvent` feature は有効でないため（ADR-0154 の制約）、
  detail ではなく **送信先別のイベント名**で区別する:
  - `openrustbooks:save-error:reader`
  - `openrustbooks:save-error:editor`
- `report_save_failure(context, target: SaveErrorTarget)` に変更:
  - `SaveErrorTarget::{Reader, Editor}`。
  - `context` は console.error の固定分類文言（下書き / 読書位置 /
    表示設定 / 履歴）にのみ使用。
  - 本文・値・キー名・生エラーはイベントへ載せない
    （ADR-0154 / 0164 の redaction 原則を維持）。
- 経路→target の対応:
  - 下書き → Editor
  - 読書位置 / 表示設定 / 履歴 → Reader
- 旧汎用名 `openrustbooks:save-error` はコードから削除。
  外部から dispatch しても status を更新しない（新規 E2E で固定）。
- E2E に 1 件追加し、期待総数 240 → **241**（check 224 + equal 17）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 送信先別イベント名（採用） | CustomEvent feature 不要で最小変更 |
| CustomEvent の detail に target | feature 追加が必要 |
| App 直下の単一 status へ集約 | 画面構成の変更が必要 |

## 影響

- 旧イベント名は後方互換なし（コード内に購読者なし）。
- `forget()` リスナーは SPA 再マウントで蓄積する既存リスクが残る
  （status 競合は解消）。
- SettingsPage の保存失敗は従来どおり Reader 宛。
  SettingsPage 専用リスナーは未追加（未決定）。
- serde / prune 失敗の一部は依然 `report_save_failure` を呼ばない
  （ADR-0154 の未決定を継続）。

## 検証（Runtime Proof）

- E2E 3 件が ok:
  - 「保存失敗時に status と console へ通知する」（Reader）
  - 「下書き保存失敗時に status と console へ通知する」（Editor）
  - 「旧共通の保存失敗イベントは status を更新しない」
- E2E **241 項目すべて成功**。
- 全単体テスト core 202 + desktop 56 + web 2、wasm-test 10、
  audit 脆弱性 0 件。
- `cargo fmt --all -- --check`、wasm32 web check、`./scripts/verify.sh`:
  すべて成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- ~~`forget()` リスナーの蓄積~~ ADR-0166 で on_cleanup + SendWrapper による解除を実装。
- App レベルの単一 status へ集約するか。
- 旧イベント名の後方互換（現状なし）。
- E2E の positive side（`:reader` dispatch で reader-status が更新される）
  と両画面同時マウントケースの追加。
- 通知対象が 3 画面以上へ増えたときのイベント名設計。
- ADR-0154 / 0164 の未決定（prune 失敗、自動保存、下書き容量、
  暗号化、複数タブ、IME / Undo）を継続。
