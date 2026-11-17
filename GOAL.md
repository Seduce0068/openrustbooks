# Goal

AI 設定変更時に実行中 service を安全に再初期化する（ADR-0106 の未決定）。

## Scope

### 背景

`update_config` は有効時に再初期化せず、`update_ai_config` は `spawn` 内の
`let _ =` でエラーを握り潰す。そのため保存済み設定（(base_url / model /
max_retries 等）と実際に送信する service が乖離し得る。

### 成果物

- `desktop/src/state/mod.rs`: 変更（fail-closed な再初期化メソッド）
- `desktop/src/commands/config.rs`: 変更（同期 → async、差分トリガー、エラー伝播）
- `docs/decisions/0107-ai-service-refresh.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `AppState` に fail-closed な再初期化メソッドを追加する。
   - 無効なら service を破棄。
   - 有効なら旧 service を破棄してから生成し、失敗時も None のままにする。
2. `update_config` / `update_ai_config` / `update_lint_config` を async 化し、
   AI または lint 設定が変わった場合だけ再初期化する。
3. 再初期化エラーは握り潰さず command の Err として返す。
4. Tauri の command 登録（`main.rs`）は変更しない。
5. テストを追加する（差分検知、無効化時の破棄など）。
6. 既存の 174 テスト、E2E 228 項目を壊さない。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- 新規依存を追加しない。
- 同意ダイアログ（ADR-0104）と送信先制限（ADR-0101）は変更しない。
- `master` へ push する。force push はしない。
