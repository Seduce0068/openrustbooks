---
type: Decision
title: "ADR-0164: エディタ下書きのローカル保存"
description: "Tauri を有効化せず、エディタの Save をブラウザ内 localStorage ドラフトとして接続する決定を記録する。"
tags:
  - ADR
  - web
  - data
generated:
  by: dsh
  at: "2026-11-18T23:00:00+09:00"
status: accepted
---

## 背景

ADR-0161 で textarea と Preview を接続したが、Save は
「desktop only / not implemented」の恒久 disabled だった。
Tauri invoke を有効化するには `withGlobalTauri` と CSP の再設計が必要で
（ADR-0128）、ブラウザ単体でも明示保存の手段を用意する。

## 決定

- 保存先は `openrustbooks:editor:{doc_id}`（localStorage、素の文字列）。
- Save ボタンを有効化し、`on:click` で現在の本文を保存。
  - 成功時: `.editor-status` に「下書きを保存しました」。
  - 失敗時: ADR-0154 の `report_save_failure("下書き")` を再利用し、
    console.error と `openrustbooks:save-error` イベントを発行。
    Editor 自身のリスナーで固定文言 `SAVE_ERROR_MESSAGE` を表示。
- 本文は localStorage の値にのみ保持し、status / console / イベント /
  属性へ出力しない。
- 表示時に下書きを読み、`set_content.set()` と
  `textarea.set_value()` の両方で復元する（ADR-0161 の
  `prop:value` 不使用契約を維持）。
- AI Generate / AI Refine / Lint / format select は従来どおり disabled。
- 自動保存は行わない（明示 Save のみ）。
- E2E を 2 件追加し、期待総数 238 → **240**
  （check 223 + equal 17）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| localStorage 明示保存（採用） | 新規依存・Tauri 設定変更なしで実現 |
| Tauri command 接続 | `withGlobalTauri` / CSP が未設計（ADR-0128） |
| 自動保存 | 容量・競合・取り消しが未設計 |

## 影響

- ADR-0161 の「Save 含む全ボタン恒久 disabled」は
  **Save draft のみ例外**へ変更（E2E 契約も更新）。
- ADR-0154 の save-error リスナーが Reader と Editor の 2 箇所になる。
  現状は SPA 遷移で排他だが、同時存在时は status が競合し得る（未決定）。
- `doc_id` が空文字だとキーが `openrustbooks:editor:` になる
  （空 ID の扱いは未決定）。
- `set_value()` がキャレット／Undo に与える影響は未検証（**推測**）。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- E2E 3 件が ok:
  - 「エディタの Save が有効でデスクトップ専用操作が disabled で理由付き」
  - 「エディタの下書きが保存されリロード後に textarea へ復元される」
  - 「下書き保存失敗時に status と console へ通知する」
- E2E **240 項目すべて成功**。
- 全単体テスト core 202 + desktop 56 + web 2、wasm-test 10、
  audit 脆弱性 0 件。
- `python3 -m py_compile`、wasm32 web check、`./scripts/verify.sh`: すべて成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 自動保存／デバウンス、下書きの容量上限・寿命・prune。
- 空 `doc_id`、`doc_id` 変更時の未保存データの扱い。
- save-error リスナーの共通化（Reader / Editor の競合解消）。
- 本文の暗号化要否（同一オリジン JS から平文で読める）。
- 複数タブ競合（ADR-0157 / 0158 から継続）。
- IME / Undo の実機検証（ADR-0161 から継続）。
