# Goal

モーダルダイアログ表示中はキーボードショートカットを停止する（todo.md B04-B07 / C01 の一部）。

## Scope

### 背景

削除確認ダイアログやショートカットヘルプが開いているときも `key_handler` が
動作するため、背景の章移動やダーク切替が起こる可能性がある。

### 成果物

- `web/src/lib.rs`: 変更（open な dialog の存在を確認するガード）
- `tools/verify-web-reader.py`: 変更（E2E 検証を追加）
- `docs/decisions/0093-modal-shortcut-guard.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `key_handler` の先頭で、開いている `<dialog>` があれば return する。
2. 既存の `?` キーはヘルプを開くため、ガードより後に処理される。
   Esc はネイティブ dialog に任せる（既存どおり）。
3. E2E で、ヘルプダイアログ表示中に `d` を押してもダーク表示が変わらない
   ことを確認する。
4. 期待 check 総数を 223 に更新する。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- 新規依存を追加しない。
- `desktop/` を変更しない。
- `master` へ push する。force push はしない。
