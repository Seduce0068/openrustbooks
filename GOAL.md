# Goal

設定保存 UI 接続の残作業を設計受容として記録する（R6 A05 の現状整理）。

## Scope

### 背景

IPC 経路（ADR-0270）と UI 起点 CAS / 構造化エラー（ADR-0269）は完了したが、
R1 監査で次が判明した:

- Web 側 SettingsView は ConfigPatch へ Serialize できず 23 フィールド不足。
- storage の allow-list DTO 複製、a11y（for/id / aria-invalid / aria-describedby）、
  `CommandError.message` の英語固定文言を日本語 status へ変換する設計が必要。
- Tauri mock E2E +1 は DTO 往復が完成していない段階では get_config 成功までしか検証できない。

### 成果物

- `docs/decisions/0271-settings-ui-scope.md`: 追加
- `docs/decisions/0253-tauri-settings-save-design.md`: 変更（命名ドリフトと残作業を明確化）
- `todo.md`: A05 は「IPC 契約完了・UI 接続は DTO 設計後」へ更新
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. 実装済み（withGlobalTauri / CAS / 構造化エラー）と未実装（DTO 往復 / a11y / E2E 成功経路 / 実 WebView）を切り分ける。
2. 不足フィールド 23 項目と storage allow-list の制約を記録する。
3. `ConfigPatch.expected` / `__TAURI__` を正とし、旧 ADR の命名は履歴として扱う。
4. E2E 255 維持、check 追加なし。
5. markdownlint 0 issues、verify.sh（255）成功。

## Acceptance criteria

- 文書・索引が整合し、全ゲート成功。

## Constraints

- web / desktop / E2E / Cargo.* は変更しない。
- 外部サービス・実 WebView 検証は行わない。
- `master` へ push する。force push はしない。

## 完了記録

- ADR-0271 で IPC 完了 / UI 未実装を分離し、23 フィールド不足の内訳を記録。
- ADR-0253 の旧命名を履歴注記、docs/index に 0269 / 0270 の導線を追加。
- コード・E2E 不変、`./scripts/verify.sh` 全ゲート成功、E2E 255 passed / 0 failed。
