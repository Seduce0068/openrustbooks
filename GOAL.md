# Goal

E2E に期待 check 総数を宣言し、未実行 check を失敗扱いにする（todo.md R4-e2e#1 / C06）。

## Scope

### 背景

`checker.check()` は条件付きブロック内で呼ばれることがあり、DOM ノードが
見つからない場合などに check 自体がスキップされる。その場合、実 passed 数が
静的な check 呼び出し数より少なくてもテストは成功してしまう。

### 成果物

- `tools/verify-web-reader.py`: 変更（期待総数の宣言と検証）
- `GOAL.md`: 本契約

### 必須事項

1. `Checker` に `expected` 引数を追加し、`finish()` で
   `passed + len(failures) == expected` を検証する。
2. 現在の期待値 220 を `run_checks()` の `Checker(...)` に設定する。
3. 乖離がある場合は失敗として報告する。
4. 既存の E2E 220 項目はすべて通る。

## Acceptance criteria

- 通常実行で 220 項目すべて成功。
- 期待値を意図的にずらした場合に失敗することを確認（任意）。

## Constraints

- 新規依存を追加しない。
- プロダクションコードを変更しない。
- `master` へ push する。force push はしない。
