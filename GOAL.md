# Goal

設定フォームを契約どおり読み取り専用にする（R6 PR-M11、ADR-0193）。

## Scope

### 成果物

- `web/src/lib.rs`: readonly/disabled、setter 削除、常時説明
- `tools/verify-web-reader.py`: E2E 1 件、249 → 250
- `docs/decisions/0193-settings-form-readonly.md`: 追加
- `todo.md`: PR-M11 を完了へ移動

### 決定

- text/number/password は readonly、checkbox/select は disabled。
- 表示用 signal と status 文言は維持。
- withGlobalTauri の問題は未決定として ADR に記録。

## Acceptance criteria

- E2E 250 項目、全ゲート成功（検証済み）。
