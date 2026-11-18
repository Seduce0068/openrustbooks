---
type: Decision
title: "ADR-0135: 秘密削除の監査ログ"
description: "clear_secrets が削除対象名だけをログに残す決定を記録する。"
tags:
  - ADR
  - security
  - config
generated:
  by: dsh
  at: "2026-11-18T11:35:00+09:00"
status: accepted
---

## 背景

ADR-0134 の未決定「削除対象名（値なし）の監査ログ」。
値をログに出すと秘密が漏れるため、対象名だけを残す必要がある。

## 決定

- `clear_targeted_secrets()` の戻り値を `bool` から
  `Vec<&'static str>` に変更し、実際に消した対象名
  （`"api_key"` / `"client_secret"` / `"refresh_token"`）のみを返す。
  順序は固定。値・パスは型レベルで混入不能。
- `clear_secrets()` は `cleared.is_empty()` なら no-op
  （ログ・save・再初期化なし）。
- `log::info!("cleared secrets: {}", cleared.join(", "))` は
  **`save_config()` 成功後・`refresh_ai_service()` の前**に出力する。
  保存失敗時に「削除済み」と記録しないため。
- `has_secret_value(Option<&str>)` を追加し、
  `None` / `Some("")` / 空 `String` をすべて「未設定」として扱う。
  `Some("")` を消しても「消えた」とは計上しない。
- 既存の `log = "0.4"` / `tauri-plugin-log` を利用（新規依存なし）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 対象名のみをログ（採用） | 監査性と秘密非開示を両立 |
| 値もログ | 秘密が漏れるため却下 |
| ログなし | 削除事実が残らない |

## 影響

- 出力例: `cleared secrets: api_key` /
  `cleared secrets: client_secret, refresh_token`。
- ログ先は `desktop/src/main.rs:19-30` の Stdout / LogDir(openrustbooks) /
  Webview、`LevelFilter::Info`。desktop の既定フォーマッタは
  タイムスタンプ・モジュール名・レベルを付与するが、
  **メッセージ本文は対象名のみ**。 suppression は他ログへ影響するため行わない。
- Webview にも生メッセージが emit されるが、対象名のみで秘密は出ない
  （`withGlobalTauri` は未設定、ADR-0134 から継続）。
- 新規依存なし、unsafe なし。

## 検証

- 追加テスト 6 件（大文字小文字ではなく対象名の完全一致、
  全 false、Drive None、既に空、全指定、mixed）。
  desktop 56 テスト（前回 50 から +6）。
- `cargo test --all-targets --locked`: core 188 + desktop 56 + web 2、
  wasm-test 3、audit 脆弱性 0 件、E2E 229 項目すべて成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定（audit-log-auditor の指摘）

- ログ保持期間・ローテーション・改ざん耐性（プラグイン既定依存）。
  真のコンプライアンス監査には別設計が必要（**推測**）。
- Webview へのログ emit を許容するか（現状は対象名のみなので実害なし）。
- `save_config` 失敗時の fail-open（メモリは消えたがディスクは残る、
  ADR-0107 / 0134 から継続）。
- in-flight 送信が古い Arc のキーを使う問題（ADR-0107 から継続）。
- Web UI 接続時の確認ダイアログ（ADR-0134 から継続）。
