# Goal

旧 DTO の往路で `max_retries` が既定値にリセットされる問題を修正する（ADR-0105 の未決定）。

## Scope

### 背景

`AiConfigView` に `max_retries` を追加したため、古い DTO JSON（キーなし）を
`update_config` へ渡すと serde の既定値 3 で保存され、利用者の設定（例 5）が
静かに消える。UI 未接続で現状は発生しないが、接続前に塞ぐ。

### 成果物

- `desktop/src/commands/config.rs`: 変更（`AiConfigView` から `max_retries` を除外、
  `merge_secrets` で現在値を保持）
- `docs/decisions/0106-retry-merge-preservation.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `AiConfigView` から `max_retries` を削除し、DTO JSON に含まれないようにする。
2. `merge_secrets` で `ai.max_retries` を現在値から保持する。
3. 既存テスト `ai_config_view_json_includes_max_retries` を
   「含まれない」検証に変更する。
4. 新規テストを追加する（マージで保持されること、DTO に無いこと）。
5. 既存の 173 テスト、E2E 228 項目を壊さない。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- 新規依存を追加しない。
- UI 配線は変更しない。
- `master` へ push する。force push はしない。
