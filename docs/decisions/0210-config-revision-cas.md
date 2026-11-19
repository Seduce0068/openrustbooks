---
type: Decision
title: "ADR-0210: 設定更新の revision CAS"
description: "生成失敗待ち中の設定更新を CAS で検出し、lost update を防ぐ決定を記録する。"
tags:
  - ADR
  - desktop
  - concurrency
generated:
  by: dsh
  at: "2026-11-19T11:23:00+09:00"
status: accepted
---

## 背景

設定コマンドは
`current = state.config()` → `merge_secrets` →
`prepare_ai_service(&merged).await` → `commit_config()` の順で動く（R6 TD-L10）。
`prepare_ai_service()` は HTTP を伴い得るため、その待ち中に他コマンドが
設定を更新すると、古い `current` に基づく更新が後勝ちで上書きされた。
AI 変更・secret 削除・reset が消え得た。
`std::sync::RwLock` を `await` を跨いで保持できないため、
read-modify-write 全体をロックで包むことはできない。

## 決定

- `AppState` の設定を `ConfigState { config, revision, pending }` として
  1 つの `RwLock` で保持する。
- `config_with_revision()` が同一 read guard で `(AppConfig, revision)` を返す。
- `commit_config(expected, config, service)` の手順:
  1. `expected.checked_add(1)` で次 revision を事前計算し、
     溢出時は保存前に `CoreError::Internal`。
  2. 短い write guard で `expected != revision` または `pending` なら
     `CoreError::Conflict("Configuration changed since it was read")` を返す。
     設定／ディスク／service は変更しない。
  3. 一致なら `pending = true` にして guard を解放する。
  4. ロック外で `AppConfig::save()` を直接呼ぶ
     （`save_config()` は read guard を取るため再ロック回避）。
  5. 保存失敗時は旧 config / revision を維持し、pending を保持したまま
     旧設定で再 save。二次失敗なら `Internal`（設定値・パス非開示）。
  6. 保存成功時のみ短い guard で config 更新・revision 前進・pending 解除、
     続けて `ai_service` を置換する（config → ai_service の一方向ロック）。
- `pending` は「guard を解放してもう一方の同じ expected が
  CAS を通過しないための予約フラグ」。
- `CoreError::Conflict(String)` を cfg なしで追加する。
  Display は固定文言のみで、設定値・パス・シークレットは含まない。
- 6 コマンド（update_config / patch / ai / lint / clear_secrets / reset_config）は
  `await` 前に `config_with_revision()` で expected を取得し、
  `commit_validated_config()` 経由で渡す。
- `ConfigView` に `revision` を追加し、
  `get_config()` / 各コマンドの返却で現在 revision を返す
  （Web の `SettingsView` は読み取り専用のままで、未知フィールドは serde が無視）。
- `clear_secrets` の `merge_secrets` 非経由、
  `reset_config` の既定値＋service クリア契約は維持する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| revision CAS（採用） | await を跨いでロックせず競合を検出できる |
| Mutex 直列化 | read から包む必要があり AI 初期化待ちが全設定操作をブロック |
| 現状維持 | AI 変更や secret 削除が消え得る |

## 影響

- 競合時は片方のコマンドが失敗する（UI 側の自動リトライは未実装）。
- 保存中 panic で pending が残る可能性はある
  （`save()` は `Result` を返すため通常経路では発生しない）。
- プロセス外の `config.toml` 直接編集は revision で検出できない（未決定）。
- wasm32 ビルドは `CoreError::Conflict` が cfg なしのため影響なし。
- 新規依存なし、unsafe なし、E2E 251 維持。

## 検証（Runtime Proof）

- unit test 6 件追加:
  - 成功 commit で revision 前進。
  - stale expected で Conflict、config / service / disk 不変、
    Display 固定文言。
  - `tokio::join!` で同一 expected の 2 更新の片方 Conflict。
  - ConfigView revision。
  - ConfigPatch が revision を無視。
  - save_config のディスク書込み。
- `cargo test --all-targets --locked`: core 261 + desktop **79** + web 2。
- `cargo clippy --all-targets --locked -- -D warnings`、
  wasm32 core / web check、`./scripts/verify.sh`: すべて rc=0。
- E2E **251 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 競合時の UI 挙動（自動リトライ／再取得／ユーザー確認）。
- プロセス外編集の検出（revision のファイル永続化が必要か）。
- revision 溢出時の挙動（現状は Internal で fail-closed）。
- 保存中 panic 時の pending 回復。
- IPC 側で Conflict を型判別するか（現状は `String` に変換）。
