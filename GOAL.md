# Goal

WDIO 実 WebView E2E に設定保存往復を追加する（R6 レビュー / R5-DC01）。

## Scope

### 背景

現在の e2e/specs/settings.e2e.ts は get_config 読み取りのみで、
ADR-0282 で手動 probe 確認した update_config_patch の成功・revision +1・
config.toml 永続化を回帰検証していない。

### 成果物

- `e2e/specs/settings.e2e.ts`: 変更
- `e2e/types.d.ts`: 必要なら変更
- `docs/decisions/0296-wdio-save-roundtrip.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. 実 WebView から update_config_patch を呼び、revision +1 を検証する。
2. config.toml の非秘密キー（例: log_level）が更新されることを検証する。
3. テスト後は元の設定値へ復元する（失敗時も finally で復元）。
4. 既存 Tauri mock E2E 256 件・check 名・WDIO 既存 spec 名を変更しない。
5. verify.sh 全ゲート成功。

## Constraints

- API key / secret / path はログ・test 出力へ出さない。
- 外部 AI API への送信は行わない。
- `master` へ push する。force push はしない。
