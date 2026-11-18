---
type: Decision
title: "ADR-0177: 設定コミットのトランザクション化"
description: "設定保存と AI サービス再初期化を準備／コミットに分け、失敗時の部分状態を防ぐ決定を記録する。"
tags:
  - ADR
  - desktop
  - config
generated:
  by: dsh
  at: "2026-11-19T03:00:00+09:00"
status: accepted
---

## 背景

設定更新コマンドは `update_config()` → `save_config()` →
`refresh_ai_service()` の順でコミットしていた。
`refresh_ai_service()` が失敗するとメモリと `config.toml` には新設定が残る一方、
AI service だけ `None` になり、利用者には失敗が返るが再起動後は新設定が有効という
中途半端な状態になっていた（R6: PR-H04）。

## 決定

- `AppState` に 2 メソッドを追加:
  - `prepare_ai_service(&AppConfig) -> CoreResult<Option<Arc<AiMarkdownService>>>`
    - 状態を変更せず、候補設定で service を生成する。
    - 無効設定は `Ok(None)`、生成失敗は `Err`（状態は不変）。
  - `commit_config(AppConfig, Option<Arc<AiMarkdownService>>) -> CoreResult<()>`
    - 旧 config を退避 → メモリ更新 → ディスク保存 → 成功時のみ service を置換。
    - 保存失敗時は旧 config を復元し、旧設定で再保存する。
    - 2 次保存失敗時は値を含まない `CoreError::internal` を返す。
- `commands/config.rs` に共通ヘルパ `commit_validated_config()` を追加。
  `merge_secrets` は各コマンド側の責任とし、共通経路には含めない。
- 対象コマンド: `update_config` / `update_config_patch` / `update_ai_config` /
  `update_lint_config` / `clear_secrets` / `reset_config`。
- no-op（`changed=false`）時は既存 service を維持する。
- `init_ai_service()` / `refresh_ai_service()` は既存 API を維持し、
  `prepare_ai_service()` を利用する（fail-closed を継続）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| prepare → commit（採用） | 生成失敗時に 3 状態が不変 |
| 途中で old service を破棄 | 失敗時に復活が必要になり fail-closed と矛盾 |
| save を最後にするだけ | 保存失敗時にメモリが新設定のままになる |

## 影響

- 生成失敗時: config / disk / service の 3 状態が不変。
  旧 service は廃棄→復活せず、同じ `Arc` が残る
  （`refresh_ai_service()` 単体は生成失敗時に `None` を維持）。
- `clear_secrets()` の `merge_secrets` 回避契約を維持。
- config / ai_service のロック同時保持なし（生成はロック外）。
- `AppConfig::save()` は固定パスへ `truncate + write` するため、
  保存途中失敗時のディスク部分破壊と 2 次 save 失敗は残る（未決定）。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- unit test 3 件追加:
  - 不正 provider で service 生成失敗 → config / service 不変。
  - 無効設定 → `Ok(None)`。
  - commit 成功で config / service 更新、無効化で service `None`。
- `cargo test --all-targets --locked`: core 210 + desktop **59** + web 2。
- wasm-test 10、audit 脆弱性 0 件。
- `cargo clippy --all-targets --locked -- -D warnings`、
  `./scripts/verify.sh`: 成功。E2E **247 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `AppConfig::save()` の一時ファイル＋rename による原子化
  （2 次 save 失敗時のディスク復旧を完全にする）。
- disk 不変の直接テスト（今回は config / service を検証）。
- lint 変更時に AI service も再初期化する理由の確認
  （`AiMarkdownService` が lint を含むか未確認、推測）。
- 並行設定更新の lost update 検出（世代カウンタ等）。
