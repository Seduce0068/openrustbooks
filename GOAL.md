# Goal

履歴が 1 件のとき localStorage が更新されないバグを修正する（todo.md B09）。

## Scope

### 背景

`save_history()` は `history.len() <= 1` のとき早期リターンするため、
履歴クリア（ADR-0031）や個別削除（ADR-0035）で 1 件になったとき、
localStorage には古い履歴が残る。再読み込みすると古い履歴が復活する。

### 成果物

- `web/src/lib.rs`: 変更（早期リターンを削除し 1 件でも保存）
- `tools/verify-web-reader.py`: 変更（E2E 検証を追加）
- `docs/decisions/0085-single-entry-history-save.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `save_history()` から `history.len() <= 1` の早期リターンを削除する。
2. 履歴が 1 件のときも、当該本の localStorage キーに
   `{ entries: [現在位置], index: 0 }` を保存する。
3. `load_history()` は従来どおり 2 件以上のみ復元する
   （1 件の保存データは次回の現在位置として扱う）。
4. E2E で、クリア後に localStorage の履歴が 1 件になることを確認する。
5. 既存の 125 テスト、E2E 219 項目を変えない。

## Acceptance criteria

- 6 ゲートすべて成功する。
- E2E が通る。

## Constraints

- 新規依存を追加しない。
- `desktop/` を変更しない。
- `master` へ push する。force push はしない。
