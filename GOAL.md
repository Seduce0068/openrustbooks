# Goal

SettingsForm を Tauri `get_config` / `update_config` に接続する（ADR-0127 の第 1 段）。

## Scope

### 背景

SettingsForm の 11 要素はすべて静的で、submit ハンドラも無い。
desktop 側には `get_config` / `update_config` が登録済みで、
DTO (`ConfigView`) は secret / パス情報を除外している。
Web 側に Tauri invoke は無い。

### 成果物

- `desktop/tauri.conf.json`: 変更（`app.withGlobalTauri: true`）
- `web/src/lib.rs`: 変更（invoke ラッパ、SettingsForm の状態と submit）
- `tools/verify-web-reader.py`: 変更（ブラウザ版で壊れないことの検証）
- `docs/decisions/0128-settings-form-wiring.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `window.__TAURI__` が無い環境（ブラウザ単体）ではエラーにせず、
   「デスクトップ版のみ利用可能」と表示する。
2. `get_config` で DTO を取得し、フォームへ初期値を設定する。
3. submit でフォーム内容を `update_config` に渡す。
   DTO から除外されたフィールド（secret / パス）は desktop 側の
   `merge_secrets` が保持するため、Web 側では送らない。
4. API Key は実値を表示せず、`has_api_key` フラグのみを表示する
   （ADR-0098 / 0103 の方針）。
5. 既存の E2E 228 項目を壊さない（期待総数の更新は必要なら行う）。
6. 新規 npm 依存は追加しない（global Tauri 経由）。

## Acceptance criteria

- 6 ゲート + audit + wasm-test + E2E が成功する。

## Constraints

- `desktop/src/commands/config.rs` の DTO / merge_secrets は変更しない。
- 新規クレートは追加しない。
- `master` へ push する。force push はしない。
