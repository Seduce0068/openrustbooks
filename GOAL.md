# Goal

Tauri 設定保存 UI を接続する（R6 A05 段階 2、ADR-0271 の DTO 要件を実装）。

## Scope

### 背景

IPC 経路（ADR-0270）と UI 起点 CAS / 構造化エラー（ADR-0269）は完了した。
web 側 `SettingsView` は `ConfigPatch` へ Serialize できず、
allow-list View の差分（general 1 / ai 6 / lint 2 / storage 14）と
`expected` が不足していた（ADR-0271）。

### 成果物

- `web/src/lib.rs`: 変更（完全な ConfigView DTO、保存 IPC、入力可能 UI、a11y）
- `tools/verify-web-reader.py`: 変更（Tauri mock 成功経路 +1、255 → 256）
- `docs/e2e-traceability.md`: 変更（256 件追従）
- `docs/documentation-operations.md`: 変更（current 256）
- `docs/decisions/0272-settings-save-ui.md`: 追加
- `todo.md`: A05 を「UI 接続完了、実 WebView 検証は後続」へ更新
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. web 側 DTO を desktop `ConfigView` / `ConfigPatch` と同形にし、
   `revision` → `expected` マッピングで保存する。
2. secret / path / extra_headers を DTO へ追加しない。
3. browser は readonly / fail-closed を維持し、desktop（IPC 成功時）のみ編集可能にする。
4. a11y: `label[for]` / `id` / `aria-invalid` / `aria-describedby` /
   `role="status" aria-live="polite"` を満たす。
5. E2E +1（成功経路）、期待総数 256、check 名一意、AST / 文書同期。

## Acceptance criteria

- E2E 256 項目、core / desktop / web / wasm / verify.sh 全ゲート成功。

## Constraints

- desktop / core / Cargo.* は変更しない（IPC 契約は ADR-0269 を正とする）。
- 実 Tauri WebView 往路検証は行わない（CDP mock のみ、未決定として残す）。
- `master` へ push する。force push はしない。
