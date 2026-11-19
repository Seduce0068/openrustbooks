# Goal

updater の未配線を設計受容とし、設定表示を正直化する（R6 TD-M21）。

## Scope

### 背景

`tauri_plugin_updater` は `desktop/src/main.rs` で初期化されるが、
更新チェックの呼び出し・公開鍵・endpoint・UI consent は未配線。
`check_updates`（既定 true）は設定画面に「有効」と表示されるだけで、
実際には何も起きない。公開鍵／署名 secrets／Release ワークフローが
未定義で、チェックだけ配線しても fail-closed で動作しない。

### 成果物

- `web/src/lib.rs`: 変更（`check_updates` 表示を「未配線」へ）
- `docs/decisions/0222-updater-deferred.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映
- `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. 設定画面の `check_updates` を true / false に関わらず「未配線」と表示する。
2. ADR に現状（初期化のみ、pubkey / endpoint なし、署名 secrets 未定義）と設計受容の根拠を記録する。
3. 将来実装する場合の最小安全設計（minisign、公開鍵、HTTPS endpoint、手動チェック、同意、CSP、Release workflow）を記載する。
4. 未決定を漏れなく記録する。
5. E2E 253 を維持し、全ゲート成功。

## Acceptance criteria

- 全ゲート成功、E2E 253 項目成功。

## Constraints

- updater の本番接続・自動チェック・自動インストールは実装しない。
- 共通ワークフロー `rust-gates.yml@v1` は変更しない。
- `master` へ push する。force push はしない。
