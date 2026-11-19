---
type: Decision
title: "ADR-0207: AppState のスコープと設計受容"
description: "設定系トランザクションの完了を確認し、AppState を composition root として設計受容する決定を記録する。"
tags:
  - ADR
  - desktop
  - config
  - architecture
generated:
  by: dsh
  at: "2026-11-19T10:24:25+09:00"
status: accepted
---

## 背景

PR-M10 で「AppState が設定・文書・AI service・保存を抱え、更新トランザクションが無い」と指摘していた
（根拠 `desktop/src/state/mod.rs:12,76,136`）。監査の結果、設定系の部分コミットは ADR-0177 の
prepare / commit で解消済みと判定した。AppState の 4 つの `RwLock` 集約は Tauri の
composition root として設計受容できるため、Repository / Store への分離は見送る。

## 決定

- 設定系トランザクションは ADR-0177（`prepare_ai_service` / `commit_config`）で完了とし、
  本 ADR では新たな実装を行わない。
- `AppState` の config / documents / ai_service / unsaved_changes（4 つの `RwLock`）を
  Tauri の composition root として設計受容する。
- Repository / Store への分離は見送り、既存 API とテスト契約を維持する。
- 監査で挙がった残リスク 3 件は低優先度 todo へ個別に切り出し、本 ADR では解消しない。

## 根拠

- 6 コマンド（`update_config` / `update_config_patch` / `update_ai_config` /
  `update_lint_config` / `clear_secrets` / `reset_config`）が
  `commit_validated_config()` を共有し、設定更新経路が一本化されている。
- 生成失敗時に config / disk / service の 3 状態が不変（ADR-0177）。
  config / service は unit 検証済み。disk の不変は構造上の帰結であり、
  直接テストは未実施（ADR-0177 の未決定として継続）。
- config と ai_service のロック同時保持はなし。service 生成はロック外で完了する。
- 分離は更新経路とテストの広範な変更を伴う。回帰リスクが便益を上回るとの判断（推測）で見送る。

| 選択肢 | 判断 |
| --- | --- |
| AppState を composition root として受容（採用） | 共有ヘルパと不変条件を維持できる |
| Repository / Store へ分離 | 回帰リスク > 便益との判断（推測）で見送り |
| 設定トランザクションの再実装 | ADR-0177 で完了済みのため不要 |

## 影響

- プロダクションコードの変更なし。設定更新の挙動と API は ADR-0177 のまま。
- PR-M10 を完了へ移動し、根拠を ADR-0177 / ADR-0207 とする。
- 残リスク 3 件を低優先度 todo（TD-L09 / TD-L10 / TD-L11）として追跡する。
- 新規依存なし、unsafe なし。

## 残リスク（低優先度で個別追跡）

- (a) `AppConfig::save()` の tmp ファイル＋rename による原子化。
  `core/src/config.rs:495-527` は固定パスへ truncate + write するため、
  保存途中失敗で config.toml が部分破壊され、2 次 save 失敗時に復旧できない（TD-L09）。
- (b) 並行設定更新の lost update 検出。
  世代カウンタ／CAS 等が無く、古い更新が新しい更新を上書きし得る（TD-L10）。
- (c) `close_document` の `unsaved_changes` 残留。
  `desktop/src/commands/document.rs:261` の `remove_document()` は
  `desktop/src/state/mod.rs:140-142` の documents のみ削除し、
  `desktop/src/state/mod.rs:176-178` の `has_unsaved_changes()` に古いフラグが残る（TD-L11）。

## 未追跡（ADR-0177 の未決定を継承）

以下は本 ADR の追跡対象外とし、ADR-0177 の未決定を引き継ぐ。

- disk 不変の直接テスト。config / service は unit 検証済みだが disk は未検証。
- lint 変更時に AI service も再初期化する根拠の確認。
  `AiMarkdownService` が lint を含むかは未確認（推測）。

## 参照

- [ADR-0177: 設定コミットのトランザクション化](0177-config-commit-transaction.md)
- [ADR-0096: AppState のテスト](0096-appstate-tests.md)
- `todo.md` の低優先度 TD-L09 / TD-L10 / TD-L11
