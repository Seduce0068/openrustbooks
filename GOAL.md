# Goal

実 Tauri WebView 検証を完了し、発見した起動ブロッカーを修正する（R6 dogfood）。

## Scope

### 背景

Tauri CLI 2.12.5 は導入済みだが、`cargo tauri dev` の実起動で次が判明した:

- `core::init()` の `tracing_subscriber` と `tauri_plugin_log` が競合し、
  "attempted to set a logger after the logging system was already initialized" で起動失敗。
- `beforeDevCommand` の `cd ../web` が CLI フックの cwd と合わず失敗。
- `tauri_plugin_updater` 2.14.1 は `plugins.updater.pubkey` を必須とし、
  設定が無いと "Error deserializing 'plugins.updater'" で起動失敗。
  updater は未配線（ADR-0222 / 0277）で、公開鍵 / endpoint / 署名は未決定。

### 成果物

- `desktop/src/main.rs`: 変更（core::init 除去、updater plugin 登録除去）
- `desktop/tauri.conf.json`: 変更（beforeDevCommand/BeforeBuildCommand の cwd 修正）
- `desktop/Cargo.toml`: updater 依存除去（必要なら Cargo.lock 追従）
- `scripts/check-updater-honesty.sh`: 変更（plugin init 要求を「未登録」要求へ反転）
- `docs/decisions/0282-updater-plugin-startup-removal.md`: 追加
- `docs/decisions/0089` / `0135` / `0222` / `0277` / `0279`: 参照更新
- `todo.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. `cargo tauri dev` で WebView ウィンドウが出現すること。
2. `/settings` が desktop の editable フォームとして表示され、
   `get_config` / `update_config_patch` が成功すること。
3. 固定成功文言と `role="status" aria-live="polite"` を確認すること。
4. config.toml の変更値のみ永続化を確認（revision はメモリ値のため再起動で 0 に戻る）。
5. updater 公開鍵 / endpoint / 署名は導入しない。ダミー pubkey は残さない。
6. E2E 256 件・check 名を変更しない。
7. markdownlint 0 issues、verify.sh 全ゲート成功。

## Acceptance criteria

- 実 WebView で設定保存が成功し、起動ブロッカーが解消されている。
- updater は未配線のまま、静的ゲートが未登録を保証する。

## Constraints

- 外部送信 / 公開ビルド / 署名 secrets / 破壊的 config 操作は行わない。
- 秘密値は表示・送信しない。
- `master` へ push する。force push はしない。
