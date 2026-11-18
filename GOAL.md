# Goal

`desktop/src/commands/document.rs` の `tokio::fs` 直呼びにパス検証を追加する（todo.md 最優先の残り）。

## Scope

### 背景

ADR-0097 で `LocalStorage::resolve_path()` を fail-closed 化したが、
`document.rs:25` (`read_to_string`) と `:102` (`write`) は backend を通さず
`tokio::fs` を直呼びする。UI 未接続のため顕在化していないが、
接続時に任意パス読書きのリスクがある。
以前は backend 経由に置換すると `local_roots` が空で回帰すると判断したが、
**backend を使わずに root 検証だけを追加する**方法なら回帰せずに守れる。

### 成果物

- `desktop/src/commands/document.rs`: 変更（パス検証ヘルパーと適用）
- `docs/decisions/0129-document-path-validation.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `AppConfig.storage.local_roots[].path` を root として、
   canonicalize + `..` 拒否 + root 配下チェックを行うヘルパーを追加する。
2. 既存の backend 経由 command の挙動は変えない。
3. `local_roots` が空の場合の挙動を決める（安全側で拒否／現状維持）。
4. テストを追加する。
5. 既存の 216 テスト + wasm 3、E2E 229 項目を壊さない。

## Acceptance criteria

- 6 ゲート + audit + wasm-test + E2E が成功する。

## Constraints

- 新規依存を追加しない。
- `core/src/io.rs` の `resolve_path()` は変更しない。
- `master` へ push する。force push はしない。
