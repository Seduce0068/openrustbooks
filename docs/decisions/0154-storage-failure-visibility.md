---
type: Decision
title: "ADR-0154: localStorage 保存失敗の可視化"
description: "localStorage への保存失敗時に固定文言の status 表示と console.error を出す決定を記録する。"
tags:
  - ADR
  - web
  - ux
generated:
  by: dsh
  at: "2026-11-18T20:10:00+09:00"
status: accepted
---

## 背景

ADR-0151 の探索で、`save_position()` / `save_settings()` / `save_history()` が
`window.local_storage()` や `set_item()` の失敗を早期 return / `let _` で
握り潰していることが判明した。容量超過縮退（ADR-0019）後も失敗した場合、
利用者も運用側も気付けなかった。

## 決定

- 共通ヘルパー `report_save_failure(context: &str)` を追加:
  - `web_sys::console::error_1` で固定の分類文言を出力。
  - `window.dispatch_event(Event::new("openrustbooks:save-error"))` で
    Reader へ通知（`web-sys` の `CustomEvent` feature は有効でないため通常の Event）。
- 利用者向け文言は固定定数
  `SAVE_ERROR_MESSAGE = "保存に失敗しました。ブラウザの設定または空き容量を確認してください。"`。
- Reader 側で `addEventListener("openrustbooks:save-error", ...)` を 1 つ登録し、
  既存の `.reader-status`（`role="status" aria-live="polite"`）へ表示。
  Closure はページ寿命保持のため `forget()`。
- 3 関数すべてで失敗時に呼び出す。
  - `save_position`: window / local_storage 取得失敗、容量超過リトライの全失敗。
  - `save_settings` / `save_history`: window / local_storage 取得失敗、
    `set_item` の `Err`（従来の `let _` を除去）。
- キー名・値・生エラー（JsValue の Debug 等）は status / console に出さない
  （ADR-0098 / 0102 の redaction 方針に準拠）。
- 保存失敗時も読書処理は中断しない（従来の方針を維持し、通知のみ追加）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| カスタムイベント＋固定文言（採用） | set_status を直接持つ保存関数から最小変更で通知可能 |
| 保存関数の Result 化 | 全呼び出し側の改修が必要で範囲が広い |
| 既存 status へ直接 Debug 出力 | 機微情報漏洩リスク |

## 影響

- E2E に 1 件追加（`Storage.prototype.setItem` を一時的に例外送出へ
  オーバーライドし、文字サイズ変更で失敗を誘発。status と console.error を確認）。
  オーバーライドは try/finally で復元し後続チェックへ非干渉。
- 期待総数を 232 → **233**（check 216 + equal 17）に更新。
- 新規依存なし。

## 検証（Runtime Proof）

- `python3 -m py_compile tools/verify-web-reader.py`、
  `cargo fmt --all -- --check`、wasm32 web check: すべて成功。
- `./scripts/verify.sh` rc=0、E2E **233 項目すべて成功**。
- 全単体テスト core 196 + desktop 56 + web 2、wasm-test 4、
  audit 脆弱性 0 件。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定（storage-visibility-auditor の指摘）

- prune 失敗（`remove_item`）を警告止まりにするか保存失敗へ格上げするか。
- `import_positions()` / `clear_reading_positions()` の部分失敗の可視化
  （ADR-0151 の import 保護不足と重なる）。
- `QuotaExceededError` を DOMException.name で分類するか、
  「保存失敗」へ統合するか（現状は統合）。
- 通知の寿命（再成功で消える／自動消滅／手動 dismiss）。
- `aria-live` の高頻度読み上げ対策（通知を dedicated signal へ分離するか）。
