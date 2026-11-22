# Goal

confirm 待ち後の AI service 陳腐化を防止する（第2cycle R6 / R2-RC01）。

## Scope

### 背景

generate / refine / convert_to_table / generate_front_matter は
confirm_send().await の前に ai_service() を取得する。
ダイアログ表示中に commit_config が AI を無効化／provider 変更すると、
古い service で外部送信される。

### 成果物

- `desktop/src/commands/ai.rs`: 変更（service 取得を confirm 後へ移動）
- `docs/decisions/0319-ai-service-post-confirm-check.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. 4 コマンドすべてで confirm_send().await 後に ai_service() を取得する。
2. 無効化された場合は固定文言で送信前に拒否する。
3. 確認ダイアログの文言 / E2E 256 件 / 既存 AI 成功契約を壊さない。
4. 単体 test で confirm 中の無効化を模して送信が行われないことを検証する（外部 API なし）。
5. verify.sh 全ゲート成功。

## Constraints

- 外部 AI API への送信 test は行わない。
- `master` へ push する。force push はしない。
