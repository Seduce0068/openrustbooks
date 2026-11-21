# Goal

バックアップ import の表示設定復元条件を明示する（R6 レビュー / R5-DC01）。

## Scope

### 背景

README は「読書位置のエクスポート・インポート（表示設定・履歴を含む）」と記載するが、
実装は現在の表示設定が localStorage に存在する場合にインポート値を復元しない。
利用者はバックアップから文字サイズ・行間・ダーク設定を復元できず、理由も分からない。

### 成果物

- `web/src/lib.rs`: 変更（import 結果のstatus に制約を明示）
- `README.md`: 変更（制約の明記）
- `docs/decisions/0302-settings-import-disclosure.md`: 追加
- `todo.md` / 索引 / log: 更新

### 必須事項

1. 現在値保護で表示設定をスキップした場合、固定文言で status に告知する。
2. 現在値が無い場合は従来どおり復元する。
3. 秘密情報 / 設定値本体は status に出さない。
4. 既存 E2E 256 件・check 名・WDIO を維持する。
5. verify.sh 全ゲート成功。

## Constraints

- 明示的な「設定を復元」UI は今回追加しない。
- 外部サービス / 秘密情報は扱わない。
- `master` へ push する。force push はしない。
