# Goal

設定 IPC で DTO 外フィールドを注入できないようにする（R6 PR-M03 / PR-M05、ADR-0186）。

## Scope

### 成果物

- `desktop/src/commands/config.rs`: normalize_to_dto_boundary / normalize_ai_from_js、キー単位ヘッダマージ、unit 3 件
- `docs/decisions/0186-config-ipc-boundary.md`: 追加
- `todo.md`: PR-M03 / PR-M05 を完了へ移動

### 決定

- update_config は ConfigPatch 境界で正規化。
- update_ai_config は extra_headers のみ遮断（api_key は ADR-0098 契約で受容）。
- merge_secrets のヘッダはキー単位（defence-in-depth）。
- E2E 249 件維持。

## Acceptance criteria

- core 220 / desktop 62 / web 2、E2E 249、全ゲート成功（検証済み）。
