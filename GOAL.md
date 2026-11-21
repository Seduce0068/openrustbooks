# Goal

未使用 Tauri plugin / 依存を削除する（R6 レビュー / R1-AR01 + R2-DP01）。

## Scope

### 背景

desktop は store / global-shortcut / shell / fs / opener / notification を登録のみで利用していない。
anyhow / tracing / tracing-subscriber / figment / thiserror / futures / objc なども未使用。
静的ゲートの必須 init 一覧もこれに合わせて見直す必要がある。

### 成果物

- `desktop/src/main.rs`: 変更（未使用 plugin 登録削除）
- `desktop/Cargo.toml`: 変更（未使用依存削除）
- `scripts/check-tauri-plugin-config.sh`: 変更（必須 init 一覧の見直し）
- `Cargo.lock`: 追従
- `docs/decisions/0303-unused-tauri-plugins.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. 実利用のない plugin 登録と依存を削除する。
2. dialog / log / wdio は維持する（設定確認ダイアログ / logger / E2E）。
3. 静的ゲートを新しい必須 init 一覧へ更新する。
4. capability に削除 plugin の権限が残っていないことを確認する。
5. verify.sh 全ゲート成功、E2E 256 件・WDIO 維持。

## Constraints

- 外部サービス / 公開設定 / 課金は扱わない。
- E2E check 名・件数を変更しない。
- `master` へ push する。force push はしない。
