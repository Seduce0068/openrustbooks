# Goal

Tauri 設定の無効な plugin scope キーを削除し、再混入を静的ゲートで防ぐ（R6 R2）。

## Scope

### 背景

`desktop/tauri.conf.json` の `plugins.fs/dialog/shell.scope.allowedPaths` は、
Tauri 2.12.5 の plugin config で有効なキーではない。
fs / shell の config は `deny_unknown_fields` で未知キーを拒否し、
dialog には config 型が無い。
現状の capability は存在せず、実効スコープは元から空である。
`desktop/gen/schemas/desktop-schema.json` は CapabilityFile 用で config 検証には使えない。

### 成果物

- `desktop/tauri.conf.json`: 変更（plugins ブロック全体を削除）
- `scripts/check-tauri-plugin-config.sh`: 追加（無効キー再混入の静的ゲート）
- `scripts/verify.sh` / `.github/workflows/ci.yml`: 変更（ゲート配線）
- `docs/decisions/0280-tauri-plugin-config-cleanup.md`: 追加
- `docs/decisions/0268-csp-consistency-gate.md` / `0278-settings-ipc-capability-scope.md`: 変更（参照更新）
- `todo.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. `plugins.fs/dialog/shell.scope.allowedPaths` を削除する。
2. plugin 初期化と独自 command 登録は維持する。
3. 静的ゲートで `plugins.*.scope` / `allowedPaths` の再混入を fail-closed で拒否する。
4. 負向 test（既知無効キー、ファイル欠落、不正 JSON）で失敗すること。
5. E2E 256 件・check 名・Cargo.* は変更しない。

## Acceptance criteria

- 負向 test が失敗し、verify.sh 全ゲート成功。

## Constraints

- capability ファイルは追加しない（ADR-0278 の判断を維持）。
- 実 Tauri WebView の起動確認は環境ブロッカーとして残す。
- `master` へ push する。force push はしない。

## 完了記録

- tauri.conf.json から無効な plugins ブロックを削除し、check-tauri-plugin-config.sh を配線。
- 負向 test 5 ケースと R4 指摘（部分文字列誤検知、古いコメント）を反映。
- verify.sh 全ゲート成功、E2E 256 passed / 0 failed。
