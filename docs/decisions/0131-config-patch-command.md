---
type: Decision
title: "ADR-0131: 設定更新用 patch command"
description: "ConfigView と同型の ConfigPatch を受け取る update_config_patch を追加し、Web UI からの安全な設定保存経路を準備する決定を記録する。"
tags:
  - ADR
  - desktop
  - config
generated:
  by: dsh
  at: "2026-11-18T10:25:00+09:00"
status: accepted
---

## 背景

ADR-0128 で SettingsForm の読み取り表示を実装したが、
`get_config` は `ConfigView`（secret / パス除外 DTO）を返す一方、
`update_config` は完全な `AppConfig` を要求する非対称性が残っていた。
DTO をそのまま送るとデシリアライズ失敗し、Web で完全再構成すると
隠しフィールドが既定値で上書きされるリスクがある。

## 決定

- `ConfigPatch`（`ConfigView` と同じ公開フィールド、`Deserialize`）を追加。
- `ConfigPatch::apply_to(&AppConfig) -> AppConfig` を追加し、
  `current.clone()` へ公開フィールドのみ上書きする。
  - 隠しフィールド 11 種は現在値を維持:
    `data_dir` / `cache_dir` / `max_retries` / `model_overrides` /
    `api_key` / `extra_headers` / `ollama.options` / `config_file` /
    `rule_config` / `recent_paths` / `local_roots[].path` /
    `client_secret` / `refresh_token`。
- `#[command] update_config_patch(state, patch) -> Result<ConfigView, String>`
  を追加し、`apply_to → merge_secrets → validate_ai_base_urls →
  差分検知 → save_config → refresh_ai_service` の既存ゲートを通す。
- 旧 `update_config` は互換性のため維持。Web UI は将来 `update_config_patch`
  を呼ぶ（`ConfigView` が返るため再 fetch 不要）。
- `has_api_key` / `has_client_secret` / `has_refresh_token` は
  **読み取り専用の存在フラグ**で、patch で受け取っても常に無視する
  （更新・削除は不可、削除は `reset_config` のみ）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| patch DTO ＋既存 current へマージ（採用） | 隠しフィールドを守りつつ公開設定を更新できる |
| DTO を update_config へ | デシリアライズ失敗 |
| Web で完全 AppConfig を再構成 | 隠しフィールド上書きリスク |

## 影響

- 既存 `update_config` と並存する。
- `local_roots` の新規 id は DTO に `path` が無いため空 `PathBuf` になる
  （ADR-0103 の id 一致ルールと微妙に異なるため未決定として残す）。
- `editor` / `reader` は完全な core 型を要求する（Web の SettingsView は
  一部フィールドのみだが、送信経路はまだ無いため実害なし）。
- 新規依存なし、unsafe なし。

## 検証

- 追加テスト 3 件（隠しフィールド保持、公開フィールド更新、
  ConfigView JSON 往復）。
- `cargo test --all-targets --locked`: core 186 + desktop 42 + web 2、
  wasm-test 3、audit 脆弱性 0 件、E2E 229 項目すべて成功。
- 途中 E2E が 1 件失敗したが、`web/dist` が古いビルドだったことが原因で、
  `trunk build --offline` 後に解消（verify.sh は一時 DIST を使うため
  最終結果は不変）。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定（config-patch-auditor の指摘）

- `apply_to` と `merge_secrets` の保持ロジックが二重化しており、
  将来のフィールド追加時に片方だけ追従するリスク。
- `has_*` を更新／削除するための dedicated command の要否。
- Web UI 接続の実装時期（`withGlobalTauri` と CSP/nonce の同時実施）。
- `local_roots` 新規 id の空 path を許容するか拒否するか。
- `google_drive: None` が「削除」として扱われる挙動の仕様化。
- 不正 base_url や service 初期化失敗の command 経由テスト。
