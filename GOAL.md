# Goal

`ConfigPatch::apply_to()` と `merge_secrets()` の隠しフィールド保持ロジックを一本化する（ADR-0131 の未決定）。

## Scope

### 背景

`apply_to()` と `merge_secrets()` はともに隠しフィールド
（secret / パス / extra_headers 等）を現在値で保持する。
重複しているため、将来フィールドを追加した際に片方だけ追従すると
経路で挙動が割れるリスクがある。

### 成果物

- `desktop/src/commands/config.rs`: 変更（共通ヘルパーへ集約）
- `docs/decisions/0132-preserve-logic-unification.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `apply_to()` は公開フィールドの上書きのみを行い、
   隠しフィールドは `merge_secrets()` に任せる。
2. `merge_secrets()` が唯一の保持ロジックになるよう整理する。
3. `local_roots[].path` の id 一致ルールを両経路で統一する
   （不一致時に空 path にするか、incoming を残すかを決める）。
4. 既存テストを維持し、必要なら追加する。
5. 既存の 230 テスト + wasm 3、E2E 229 項目を壊さない。

## Acceptance criteria

- 6 ゲート + audit + wasm-test + E2E が成功する。

## Constraints

- 新規依存を追加しない。
- 既存 command の外見上の挙動は変えない。
- `master` へ push する。force push はしない。
