# Goal

設定保存と AI サービス再初期化をトランザクション化する（R6 PR-H04、ADR-0177）。

## Scope

### 成果物

- `desktop/src/state/mod.rs`: prepare_ai_service / commit_config、unit 3 件
- `desktop/src/commands/config.rs`: 6 コマンドを新 API 化
- `docs/decisions/0177-config-commit-transaction.md`: 追加
- `todo.md`: PR-H04 を完了へ移動

### 決定

- 生成失敗時は config / disk / service の 3 状態が不変。
- 保存成功後にのみ service を置換。
- clear_secrets の merge_secrets 回避契約を維持。
- save の原子化は未決定として ADR に記録。

## Acceptance criteria

- E2E 247 項目、全ゲート成功（検証済み）。
