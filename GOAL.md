# Goal

desktop 設定画面から API キーを設定・削除可能にする（第2cycle R6 最高優先 / R5-DC01）。

## Scope

### 背景

API Key 欄は常に readonly で `has_api_key` 表示のみ。
`update_ai_config` / `clear_secrets` command は存在するが web から呼ばれていない。
そのため初回インストール後に UI から AI を使い始められない。

### 成果物

- `desktop/src/commands/config.rs`: 必要なら専用 command / DTO 追加
- `web/src/lib.rs`: 変更（秘密入力 UI + invoke wrapper）
- `docs/decisions/0315-secret-settings-ui.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. desktop のみ API キー入力欄を編集可能にし、保存 command を呼ぶ。
2. 「秘密を削除」ボタンで `clear_secrets` を呼ぶ。
3. 秘密値は ConfigView / status / log / DOM の永続表示へ出さない（`has_*` と固定文言のみ）。
4. ブラウザでは従来どおり readonly / 非接続を維持。
5. 既存 E2E 256 件・check 名・#settings-language / updater / 保存文言を維持。
6. verify.sh 全ゲート成功、WDIO ゲート維持。

## Constraints

- API キー以外の秘密（client_secret / refresh_token）の UI 編集は行わない。
- 外部 AI API への送信テストは行わない。
- `master` へ push する。force push はしない。
