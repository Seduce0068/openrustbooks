# Goal

dead config を配線または明示する（R6 レビュー / R1-ST01 + R3-AU01）。

## Scope

### 背景

設定 DTO には未配線フィールドが残る:
Ollama keep_alive_minutes / LocalRootConfig.watch / default_backend /
auto_save_delay / model_overrides / config_file / telemetry / max_recent /
max_tokens_per_month。保存・復元はされるが動作に反映されない。

### 成果物

- `core/src/config.rs`: 変更（未配線フィールドの扱いを明確化）
- `desktop/src/commands/config.rs`: 必要なら変更
- `docs/decisions/0304-dead-config-cleanup.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. 各未配線フィールドを「配線」または「DTO から除外」のどちらかに 결정する。
2. 除外する場合、既存 config.toml の互換読み込みは壊さない（serde default 等）。
3. JS DTO に未配線フィールドを露出しない。
4. 既存 E2E 256 件・check 名・WDIO を維持。
5. verify.sh 全ゲート成功。

## Constraints

- 新機能の実装（watch / telemetry 等）は今回の範囲外。
- 外部サービス / 秘密情報は扱わない。
- `master` へ push する。force push はしない。
