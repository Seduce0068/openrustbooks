# Goal

web の設定保存と autosave の lost update を修正する（R6 レビュー / R2-BV01）。

## Scope

### 背景

- SettingsForm は保存開始時の view をスナップショットし、成功時に config signal 全体を
  updated で置換するため、保存 in-flight 中の編集が失われる。
- editor の autosave_last_text は手動保存時に更新されず、doc_id 変更時にも
  クリアされないため、別文書 / 手動保存後の本文が自動保存でスキップされる。

### 成果物

- `web/src/lib.rs`: 変更
- `docs/decisions/0300-lost-update-guard.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. settings 保存中に編集された場合、成功レスポンスでローカル編集を上書きしない。
2. 保存中は入力を無効化するか、dirty フラグで再保存を促す。
3. editor の last_text を doc_id 変更時と手動保存時に同期する。
4. 既存 E2E 256 件・check 名・WDIO ゲートを維持する。
5. verify.sh 全ゲート成功。

## Constraints

- 外部 AI API / 秘密情報は扱わない。
- Leptos の既存 signal 契約を大きく変えない。
- `master` へ push する。force push はしない。
