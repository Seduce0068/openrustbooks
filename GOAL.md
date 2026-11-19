# Goal

設定保存 IPC に UI 起点の CAS と構造化エラーを追加する（R6 A05 実装前提 3）。

## Scope

### 背景

`update_config_patch` は `expected` をコマンド内で再取得するため、
UI が `get_config` で読んだ revision を CAS に渡せず、stale な画面内容が
別更新を上書きし得る。また全設定コマンドが `Result<T, String>` で、
Conflict / Validation / Storage を機械的に判別できない。
ADR-0253 の実装前提 3 は、この 2 点を解消することを求めている。

### 成果物

- `core/src/error.rs`: 変更（`CoreError::code()` を追加）
- `desktop/src/commands/config.rs`: 変更（`ConfigPatch.expected` 必須化、`CommandError { code, message }`、`update_config_patch` の契約変更、test 追加）
- `docs/decisions/0269-ipc-cas-contract.md`: 追加
- ADR-0131 / 0210 / 0253 / 0254: 変更（契約更新を記録）
- `todo.md`: A05 の第三段完了、IPC 経路決定と UI 接続は後続
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. `ConfigPatch` に必須 `expected: u64` を追加し、
   `update_config_patch` がそれを `commit_validated_config` へ渡す。
2. `CommandError { code, message }` を追加し、code は
   `CONFIG_CONFLICT` / `CONFIG_VALIDATION` / `CONFIG_STORAGE` / `CONFIG_INTERNAL` に固定する。
3. 構造化エラーの message は固定文言のみで、設定値 / パス / 秘密情報を出さない。
4. CAS 成功 / stale conflict / validation / storage の regression test を追加する。
5. 全ゲート成功（E2E 255 維持、web / E2E は変更しない）。

## Acceptance criteria

- core / desktop test 成功、E2E 255、全ゲート成功。

## Constraints

- web / E2E / IPC 経路（withGlobalTauri 等）は変更しない（ADR-0264 の未決定を維持）。
- `update_config` / `reset_config` / `clear_secrets` は既存 String エラーを維持。
- `master` へ push する。force push はしない。

## 完了記録

- ConfigPatch.expected 必須化と CommandError { code, message } を追加。
- 監査指摘を反映し message は固定文言のみ、stale expected は CONFIG_CONFLICT。
- core 287 / desktop 82 / web 2 / wasm 14、E2E 255/0、全ゲート成功。
