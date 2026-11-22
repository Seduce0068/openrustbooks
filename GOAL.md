# Goal

Config DTO の desktop / web 契約を静的 test で保証する（第2cycle R6 / R1-DS01）。

## Scope

### 背景

ConfigView / ConfigPatch が desktop と web で手書き重複しており、
フィールド削除・改名・型変更がコンパイル時に検出されない。
実行時に deserialize 失敗 / 未知フィールドの黙った値落ちになるリスクがある。

### 成果物

- `tests fixtures` または `scripts`: 変更
- `desktop/src/commands/config.rs`: 変更（fixture 生成 / contract test）
- `web/src/lib.rs`: 変更（fixture deserialize test）
- `scripts/check-config-dto-contract.sh`: 追加（必要なら）
- `docs/decisions/0322-config-dto-contract.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. desktop ConfigView の JSON を単一 fixture として出力または埋め込み、web SettingsView が欠落なく deserialize できることを test する。
2. web → desktop の ConfigPatch も同じフィールド集合であることを test する。
3. フィールド不一致時に CI / verify.sh が fail-closed になる。
4. E2E 256 件・check 名・既存 settings 契約を維持。
5. verify.sh 全ゲート成功。

## Constraints

- 新規依存は使わない。
- core / wasm の既存 DTO 契約を壊さない。
- `master` へ push する。force push はしない。

## 完了記録

- Config DTO 12 ペアの静的比較ゲートを verify.sh / CI へ配線。
- revision / expected の片側例外のみ許容、負向 7 ケースを検出。
- desktop 165 / web 19 tests、verify.sh 全ゲート成功（E2E 256、WDIO 1）。
- 型トークン比較 / deny_unknown_fields は ADR-0322 の未決定。
