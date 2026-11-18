# Goal

設定フォームを契約どおり読み取り専用にする（R6 PR-M11）。

## Scope

### 背景

`SettingsForm()` は読み取り表示のみの契約（ADR-0128）にもかかわらず、
チェックボックス／テキスト入力が `on:input` / `on:change` で signal を更新し、
利用者は変更が効いたと誤解する。保存ボタンは disabled で、
ブラウザ版では設定を永続化できない。

### 成果物

- `web/src/lib.rs`: 変更（入力要素の readonly / disabled、説明文追加、signal 更新除去）
- `tools/verify-web-reader.py`: 変更（E2E 1 件、249 → 250）
- `docs/decisions/0193-settings-form-readonly.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. SettingsForm の全入力要素を読み取り専用にする
   （テキスト / number は `readonly`、checkbox / select は `disabled`）。
2. `on:input` / `on:change` による signal 更新を削除し、表示用 signal のままにする。
3. ページ先頭に「ブラウザ版は表示のみ／保存はデスクトップ版のみ」の説明を表示する。
4. E2E 1 件追加（設定フォームの入力が readonly/disabled で、説明文が表示される）。
   期待総数 249 → 250。
5. 全ゲート成功。

## Acceptance criteria

- E2E 250 項目、全ゲート成功。

## Constraints

- 設定 DTO / Tauri command は変更しない。
- 既存の表示内容・見出し・status 文言は維持する。
- `master` へ push する。force push はしない。
