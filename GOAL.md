# Goal

削除確認ダイアログを開いたとき、安全な操作（Cancel）にフォーカスが当たることを E2E で検証する。

## Scope

### 背景

ADR-0047 の未決定「ダイアログのフォーカスが Cancel に当たっていることの明示的な検証」。
`autofocus` 属性を付けているが、ネイティブ `<dialog>` の `showModal()` では
効かないケースもあるため、実際のフォーカスを確認する。

### 成果物

- `tools/verify-web-reader.py`: 変更（フォーカス検証を追加）
- `docs/decisions/0048-dialog-focus-test.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. E2E 検証:
   - 削除ボタンでダイアログを開いた直後、`document.activeElement` が
     Cancel ボタンであること。
   - プロダクションコードは変更しない（現在の実装で正しく動作している）。
2. 既存の 119 テスト、E2E 159 項目を変えない。

## Acceptance criteria

- 6 ゲートすべて成功する。
- CI は billing issue で実行不可（ローカル検証で代替）。

## Constraints

- 新規依存を追加しない。
- プロダクションコードを変更しない。
- `master` へ push する。force push はしない。
