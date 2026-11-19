# Goal

AI コマンドの confirmation 異常系と入力変換の不正値を unit test で固定する（R6 TD-L06、ADR-0202）。

## Scope

### 成果物

- `desktop/src/commands/ai.rs`: interpret_confirmation / parse_table_format / parse_table_alignment + test 10 件
- `docs/decisions/0202-ai-command-error-tests.md`: 追加
- `todo.md`: TD-L06 を完了へ移動

### 決定

- AppHandle 非依存の純粋関数へ抽出し、既存文言と大小文字/未知値挙動を維持。
- 4 コマンドの confirm_send 呼び出しと format/alignment の順序を維持。

## Acceptance criteria

- desktop 72 tests / E2E 251、全ゲート成功（検証済み）。
