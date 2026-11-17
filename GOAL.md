# Goal

desktop の `AppState` 状態遷移をテストする（test-scanner #5）。

## Scope

### 背景

`desktop/src/state/mod.rs`（111 行）にテストが無く、
`is_unsaved` / `has_unsaved_changes` は未保存クローズ拒否を支配する重要な状態。
Tauri ランタイムなしで `AppState::new` から状態遷移を検証できる。

### 成果物

- `desktop/src/state/mod.rs`: 変更（#[cfg(test)] テストを追加）
- `GOAL.md`: 本契約

### 必須事項

1. 次を検証するテストを追加する。
   - `AppState::new` 直後は未保存変更なし。
   - `add_document` → `mark_unsaved` で `is_unsaved` / `has_unsaved_changes` が true。
   - `mark_saved` で false に戻る。
   - `remove_document` で該当 id が消え、`has_unsaved_changes` が false。
   - `update_config` / `config` の往復。
2. 新規依存を追加しない。
3. 既存の 126 テスト、E2E 223 項目を壊さない。

## Acceptance criteria

- `cargo test --all-targets --locked` が新規テストを含めて成功。
- 6 ゲート・E2E が成功する。

## Constraints

- `desktop/src/main.rs` や Tauri コマンドは変更しない。
- `master` へ push する。force push はしない。
