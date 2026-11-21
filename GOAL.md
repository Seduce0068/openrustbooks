# Goal

refresh_ai_service の設定競合を修正する（R6 レビュー / R2-BV01）。

## Scope

### 背景

refresh_ai_service は disable → config 取得 → prepare_ai_service().await → 代入の順で動く。
await 中に commit_config が新 config / revision / service を確定すると、
古い config から生成した service が後から上書きされ、config と service が不一致になる。

### 成果物

- `desktop/src/state/mod.rs`: 変更
- `docs/decisions/0299-ai-service-refresh-race.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. refresh_ai_service は開始時 revision を記録し、await 後に再確認する。
2. revision が変わっていれば生成結果を捨て、現在 config と service を保持する。
3. commit_config の CAS / pending / service 不可分性を壊さない。
4. 単体 test で競合時に古い service が上書きされないことを検証する（外部 AI API なし）。
5. verify.sh 全ゲート成功、E2E 256 件・WDIO 維持。

## Constraints

- 外部 AI API への送信は行わない。
- 既存 enable/disable/ai_service 契約を壊さない。
- `master` へ push する。force push はしない。

## 完了記録

- refresh_ai_service が開始/終了で revision を確認し、競合時に古い service を破棄。
- config write guard で確認〜置換を不可分化。
- 競合再現 test 3 件、desktop 124 tests、verify.sh 全ゲート成功（E2E 256、WDIO 1）。
