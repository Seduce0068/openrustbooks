# Goal

残存する `__prune_*` デバッグコードを削除する（todo.md B10 / C08 の一部）。

## Scope

### 背景

`web/src/lib.rs` の `prune_positions()` に `__prune_before` / `__prune_after` を
`window` へ書き込むデバッグコードが残っている。E2E からは参照されておらず、
本番コードに不要な副作用（グローバルへの書き込み）がある。

### 成果物

- `web/src/lib.rs`: 変更（デバッグコード削除）
- `GOAL.md`: 本契約

### 必須事項

1. `__prune_before` / `__prune_after` の書き込みを削除する。
2. 掃除ロジック本体（`prune_positions()` の動作）は変更しない。
3. 既存の 126 テスト、E2E 220 項目を変えない。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- 新規依存を追加しない。
- `desktop/` を変更しない。
- `master` へ push する。force push はしない。
