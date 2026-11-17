# Goal

AI を無効化した後に service が残存し送信できる問題を修正する（ai-consent-auditor 経路 B/C）。

## Scope

### 背景

- `AppState::ai_service()` は `config.ai.enabled` を確認せず、service の有無だけを見る。
- 設定で `enabled=false` に戻しても `ai_service` が `None` に戻される箇所が無い。
- そのため、一度有効化された後は設定上無効でも AI コマンドが送信できる。

### 成果物

- `desktop/src/state/mod.rs`: 変更（ai_service() ガード、disable メソッド、テスト）
- `desktop/src/commands/config.rs`: 変更（無効化時に service を破棄）
- `docs/decisions/0100-ai-service-guard.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `AppState::ai_service()` は `config.ai.enabled == false` のとき None を返す。
2. `update_config` / `update_ai_config(enabled=false)` / `reset_config` のとき
   `ai_service` を None に戻す。
3. AI コマンドのシグネチャ・Tauri 登録は変更しない。
4. 既存の 143 テスト、E2E 228 項目を壊さない。
5. 新規テストを追加する。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- 同意ダイアログ UI は今回行わない（将来タスク）。
- 新規依存を追加しない。
- `master` へ push する。force push はしない。
