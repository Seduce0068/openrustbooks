# Goal

`desktop/src/main.rs` の `unwrap()` を Result 化する（todo.md C05 の一部）。

## Scope

### 背景

`build_app().setup()` 内の `app.get_webview_window("main").unwrap()` は、
メインウィンドウが見つからない場合にパニックする。セットアップ失敗を
Result として返し、Tauri 側でエラーとして扱う方が安全。

### 成果物

- `desktop/src/main.rs`: 変更（unwrap → Result）
- `GOAL.md`: 本契約

### 必須事項

1. `get_webview_window("main")` の `unwrap()` を削除し、
   `ok_or_else(...)?` で `Box<dyn Error>` に変換する。
2. 既存の `setup_window` / AI 初期化の挙動は変えない。
3. 6 ゲート・E2E が成功する。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- 新規依存を追加しない。
- Web 版には影響しない。
- `master` へ push する。force push はしない。
