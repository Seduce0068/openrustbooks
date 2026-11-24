---
type: Decision
title: "ADR-0107: AI 設定変更時の service 再初期化"
description: "AI/lint 設定変更時に service を fail-closed で再初期化する決定を記録する。"
tags:
  - ADR
  - ai
  - desktop
generated:
  by: dsh
  at: "2026-11-18T04:10:00+09:00"
status: accepted
---

## 背景

`update_config` は有効時に service を再初期化せず、`update_ai_config` は
`spawn` 内の `let _ =` でエラーを握り潰していた。そのため保存済み設定と
実際に送信する service が乖離し得た（ADR-0106 の未決定、
service-refresh-auditor の指摘）。

## 決定

- `AppState::refresh_ai_service()` を追加する。
  - 無効なら `disable_ai_service()` を呼ぶ。
  - 有効なら旧 service を破棄してから `init_ai_service().await` を呼ぶ。
  - 生成失敗時も service は None のまま（fail-closed）。
- `update_config` / `update_ai_config` / `update_lint_config` / `reset_config`
  を async 化し、merge → validate → save の後、`ai` または `lint` 設定が
  変わった場合だけ `refresh_ai_service().await` を呼ぶ。
- 差分検知は `serde_json::to_value` を `match` で比較する
  （`Result` を直接 `!=`  comparison できないため）。
- `spawn` + `let _ =` を削除し、エラーは command の Err として返す。
- Tauri の command 登録（`main.rs`）と JS 側 invoke 引数は不変。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 差分トリガー＋async await（採用） | 無関係な保存で再ビルドせず、順序とエラーが明確 |
| 常に再初期化 | 軽量だがゼロコストではない |
| fire-and-forget spawn | エラーが見えず、保存成功後に旧 service が残る |

## 影響

- AI / lint 設定を変更すると、その応答が返る前に service が作り直される。
- 再初期化失敗時は service が None になり、AI 操作はエラーになる
  （古い設定で送信し続けるより安全）。
- 新規依存は無い。

## 検証

- 追加テスト 5 件（ai 差分、lint 差分、general/reader/storage は非対象、
  無効時 None、初期化失敗の伝播と None 維持）。
- `cargo test --all-targets --locked`: core 151 + desktop 28、すべて成功。
- `./scripts/verify.sh` を実行し、6 ゲート・trunk build・WASM パス検査・
  E2E 228 項目がすべて成功した。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 解決済み

- `core/prompts/*.hbs` 全 7 ファイルに `system_prompt` が無く、native の
  `AiMarkdownService::new()` が設定に関わらず常に失敗する問題は ADR-0108 で
  全テンプレートの front matter を修正して解決済み。

## 未決定（service-refresh-implementer / async-command-auditor の指摘）

- in-flight の送信はスナップショットされた `Arc<AiMarkdownService>` を使う
  ため、同意ダイアログ表示中の設定変更は当該送信に反映されない。
  AI を無効化しても実行中の送信は完了する。
- `save_config()` 失敗時は refresh に到達せず、メモリ上 config と service が
  乖離する（既存挙動）。
