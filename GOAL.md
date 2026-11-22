# Goal

TableAlignment の未知値を fail-closed 化する（第2cycle R6 / R1-DS01）。

## Scope

### 背景

parse_table_format は未知値を CONFIG_VALIDATION で拒否する一方、
parse_table_alignment は未知値を Left へ黙ってフォールバックする。
クライアントの誤記が要求と異なる表を生成しても通知されない。

### 成果物

- `desktop/src/commands/ai.rs`: 変更
- `docs/decisions/0324-table-alignment-validation.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. parse_table_alignment を Result 化し、未知値を fixed code の validation エラーで拒否する。
2. None は従来どおり None を返す。
3. 大文字小文字無視・既存 left/center/right の成功契約を維持。
4. 単体 test を更新し、未知値拒否を固定する。
5. verify.sh 全ゲート成功、E2E 256 件・WDIO 維持。

## Constraints

- 外部 AI API への送信 test は行わない。
- `master` へ push する。force push はしない。
