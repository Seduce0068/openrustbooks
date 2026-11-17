# Goal

AI 機能の既定を無効にし、同意なく本文が送信されないようにする（todo.md D06 の一部）。

## Scope

### 背景

`core/src/config.rs` の `AiConfig::default()` は `enabled: true` で、
`desktop/src/main.rs` の setup で起動時に `init_ai_service()` が呼ばれる。
UI 未接続のため実際の送信は起きていないが、機能を接続した瞬間に
同意なしで本文が送られるリスクがある。

### 成果物

- `core/src/config.rs`: 変更（既定 `enabled: false`）
- `desktop/src/state/mod.rs`: 変更（テストの期待値を更新）
- `docs/decisions/0099-ai-default-off.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `AiConfig::default().enabled` を false にする。
2. 既存設定ファイルで明示的に true が設定されていれば従来どおり有効。
3. 起動時の `init_ai_service()` は `is_ai_enabled()` ガードのまま（既定では呼ばれない）。
4. 既存テストを更新し、新規テストを追加する。
5. 6 ゲート・E2E が成功する。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- AI 送信プレビュー／同意ダイアログの UI 実装は今回行わない（将来タスク）。
- OS キーチェーン移行は今回行わない。
- `master` へ push する。force push はしない。
