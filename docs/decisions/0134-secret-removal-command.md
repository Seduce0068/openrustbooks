---
type: Decision
title: "ADR-0134: 秘密情報の個別削除コマンド"
description: "reset_config を使わずに API キー等だけを削除する clear_secrets コマンドを追加する決定を記録する。"
tags:
  - ADR
  - security
  - config
generated:
  by: dsh
  at: "2026-11-18T11:25:00+09:00"
status: accepted
---

## 背景

ADR-0131 / 0132 で `has_*` は読み取り専用の存在フラグとなり、
`ConfigPatch` で受け取っても無視される。また `merge_secrets` は
`None` / 空 / `"***"` を「未変更」として現在値を復元するため、
通常の更新経路では秘密情報を消せなかった。
`reset_config` は全設定を既定に戻すため、他設定が失われる。

## 決定

- `SecretTargets { api_key: bool, client_secret: bool, refresh_token: bool }`
  を追加（`#[serde(default)]` で欠落フィールドは false＝fail-closed）。
- `#[command] clear_secrets(state, targets) -> Result<ConfigView, String>` を追加。
  - `state.config()` のフルクローンを起点に、指定されたフィールドだけを
    `None` / 空文字にする（他設定は不変）。
  - **意図的に `merge_secrets` を通さない**（通すと削除値が現在値として復元される）。
    この例外をコードコメントに明記。
  - 実際に値を消した場合のみ `update_config` → `save_config` →
    `refresh_ai_service` を実行。全 false / `google_drive: None` /
    既に空のケースは no-op（save・再初期化なし）。
  - `refresh_ai_service` は古いキーを保持する Arc を破棄するために必須。
- Web UI への接続は行わない（Tauri command 登録のみ）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| dedicated command（採用） | 他設定を保ったまま秘密だけ消せる |
| reset_config | 全設定が失われる |
| ConfigPatch の has_* を利用 | presence フラグで削除意図を表せず危険 |

## 影響

- IPC 公開面が 1 つ増える（`main.rs` の `generate_handler!` に登録）。
- `withGlobalTauri` は未設定のため、現状 Web UI からは呼べない。
- 新規依存なし、unsafe なし。

## 検証

- 追加テスト 6 件（api_key のみ削除、全 false no-op、`google_drive: None`
  安全性、Drive シークレット削除時の他フィールド不変、既に空なら no-op、
  JSON 欠落 false）。
- `cargo test --all-targets --locked`: core 188 + desktop 50 + web 2、
  wasm-test 3、audit 脆弱性 0 件、E2E 229 項目すべて成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定（secret-clear-auditor の指摘）

- no-op 判定は「指定フラグ」基準で、既に空の値を指定した場合も
  ヘルパーが false を返すため実質 no-op（command の早期 return テストは
  純粋ヘルパーでカバー）。
- `save_config` 失敗時にメモリ上は消えたがディスクに残る fail-open
  （ADR-0107 から継続）。
- Drive の client_secret だけを消すと refresh_token が残る意味、
  プロバイダ側の revoke は行わない（ローカル設定のみ）。
- in-flight 送信が古い Arc のキーを使う問題（ADR-0107 から継続）。
- Web UI 接続時に確認ダイアログを必須化する（ADR-0104 の送信同意とは別）。
- 削除対象名（値なし）の監査ログ。
- `google_drive: None` の意味の仕様化（ADR-0131 / 0132 から継続）。
