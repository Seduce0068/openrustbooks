# Goal

設定 IPC で DTO 外フィールド（`extra_headers` 等）を JS から注入できないようにする（R6 PR-M03 / PR-M05）。

## Scope

### 背景

`update_config` は `AppConfig` 全体を受け取るため、Web UI 向け DTO で
除外されている `extra_headers` を JS から任意に送り込める。
`are_headers_placeholder()` は `any` のため、1 つでもマスク値があると
ヘッダ全体の変更が捨てられる（PR-M05）。

### 成果物

- `desktop/src/commands/config.rs`: 変更（IPC 境界の正規化とヘッダ単位マージ）
- `docs/decisions/0186-config-ipc-boundary.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. `update_config` が受け取った生 `AppConfig` について、
   DTO 外／機微フィールド（`extra_headers`、ローカルパス、秘密値）を
   現在値または安全な既定値へ正規化してから `merge_secrets()` に渡す。
2. `merge_secrets()` は既存契約を維持しつつ、
   `extra_headers` はキー単位でマスク値のみ現在値を復元し、
   それ以外の変更は反映する（PR-M05）。
3. `update_config_patch()` の既存契約を壊さない（ConfigPatch 経路が安全なまま）。
4. unit test を追加:
   - JS から任意 `extra_headers` を送っても保存されない。
   - マスク値キーは現在値を維持し、別キーの変更は反映される。
5. 全ゲート成功（E2E 249 維持）。

## Acceptance criteria

- 全ゲート成功、E2E 249 項目成功。

## Constraints

- 新規依存を追加しない。
- 既存 `ConfigPatch` / `ConfigView` の DTO 契約を壊さない。
- `master` へ push する。force push はしない。
