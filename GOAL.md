# Goal

並行設定更新の lost update を revision CAS で検出する（R6 TD-L10、ADR-0177 / 0207 / 0209 の残リスク）。

## Scope

### 背景

設定コマンドは `current = state.config()` → `merge_secrets` →
`prepare_ai_service().await` → `commit_config()` の順で動く。
`await` 中に他コマンドが設定を更新すると、古い `current` に基づく更新が
後勝ちで上書きされ、AI 変更や secret 削除、reset が消え得る。
std::sync::RwLock を await を跨いで保持できないため、
read-modify-write 全体をロックで包むことはできない。

### 成果物

- `core/src/error.rs`: 変更（`CoreError::Conflict(String)` 追加）
- `desktop/src/state/mod.rs`: 変更（revision 保持、`config_with_revision()`、
  `commit_config(expected, config, service)`）
- `desktop/src/commands/config.rs`: 変更（6 コマンドが expected revision を伝播）
- `docs/decisions/0210-config-revision-cas.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. AppState が config と revision を同じ RwLock で保持し、
   `config_with_revision()` で (AppConfig, u64) を返す。
2. `commit_config(expected, config, service)` が expected != current なら
   `CoreError::Conflict` を返し、config / disk / service を変更しない。
3. 保存成功時に revision を 1 進める。保存失敗・rollback 時は進めない。
4. 6 コマンド（update_config / patch / ai / lint / clear_secrets / reset_config）が
   read 時に取得した expected を commit へ渡す。
5. `reset_config` も現在 revision を取得してから既定値をコミットする。
6. unit test を追加:
   - 正常 commit で revision が進む。
   - stale expected で Conflict になり状態不変。
   - `tokio::join!` 等で 2 更新の片方が Conflict になる。
7. 全ゲート成功（E2E 251 維持）。

## Acceptance criteria

- 全ゲート成功、E2E 251 項目成功。

## Constraints

- await を跨いで std::sync::RwLock のガードを保持しない。
- 既存の merge_secrets / clear_secrets 契約を壊さない。
- IPC のエラー文言に設定値・秘密・パスを出さない。
- `master` へ push する。force push はしない。
