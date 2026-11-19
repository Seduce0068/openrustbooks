# Goal

AI コマンドの confirmation 異常系と入力変換の不正値を unit test で固定する（R6 TD-L06）。

## Scope

### 背景

`desktop/src/commands/ai.rs` の `confirm_send()` は
`Ok(false)` / `Err(_)` 分岐を持つが unit test が無い。
`convert_to_table()` の不正 format、未知 alignment の Left フォールバックも未検証。
`AppHandle` 依存を避けるため、純粋な変換／結果解釈関数を抽出する。

### 成果物

- `desktop/src/commands/ai.rs`: 変更（純粋関数抽出と unit test）
- `docs/decisions/0202-ai-command-error-tests.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. `confirm_send()` の `rx.await` 結果を解釈する純粋関数（例 `interpret_confirmation(operation, Result<bool, RecvError>)`）を抽出し、3 分岐をテストする。
2. `convert_to_table()` の format 解釈を純粋関数へ抽出し、csv / json / tsv / 不正値をテストする。
3. alignment 解釈を純粋関数へ抽出し、left / center / right / 未知値の Left フォールバックをテストする。
4. 既存の Tauri コマンド挙動・エラー文言を維持する。
5. 全ゲート成功（E2E 251 維持）。

## Acceptance criteria

- 全ゲート成功、E2E 251 項目成功。

## Constraints

- 新規依存を追加しない。
- Tauri dialog の実呼び出しは変更しない（待ち受け部分のみ抽出）。
- エラー文言に本文・URL を追加しない。
- `master` へ push する。force push はしない。
