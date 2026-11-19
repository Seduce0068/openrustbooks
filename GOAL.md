# Goal

PR-M10 を「設定系トランザクションは ADR-0177 で解消、AppState は設計受容」と整理し、残リスクを個別タスク化する。

## Scope

### 背景

`todo.md` の PR-M10「AppState が設定・文書・AI service・保存を抱え更新トランザクションが無い」は、
ADR-0177 で `prepare_ai_service()` / `commit_config()` による設定系トランザクションが
実装済みにもかかわらず残っている。
一方、AppState の 4 RwLock 集約は Tauri の composition root として妥当であり、
大規模な Repository / Store 分離は回帰リスクが便益を上回る。

### 成果物

- `docs/decisions/0207-appstate-scope.md`: 追加
- `docs/decisions/0177-config-commit-transaction.md`: 未決定の参照を更新
- `todo.md`: PR-M10 を完了へ移動し、残リスク 3 件を低優先度で追加
- `docs/index.md` / `README.md` / `docs/log.md`: 更新
- `GOAL.md`: 本契約

### 必須事項

1. 設定系トランザクションが ADR-0177 で実装済みであることを根拠付きで記録する。
2. AppState の composition root としての設計受容を明記する。
3. 残リスクを 3 件に整理して todo へ追加:
   - `AppConfig::save()` の tmp + rename 原子化
   - 並行設定更新の lost update 検出
   - `close_document()` の `unsaved_changes` 残留
4. markdownlint が通る。コード変更なし（E2E 251 維持）。

## Acceptance criteria

- markdownlint 0 issues、既存ゲートを壊さない。

## Constraints

- コードは変更しない（本次は design record / todo 整理）。
- 既存 ADR の本文は履歴として保持し、相互参照のみ追加する。
- `master` へ push する。force push はしない。
