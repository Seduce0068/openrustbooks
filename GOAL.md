# Goal

rumdl auto-fix の未収束情報を保持し、fail-closed にする（ADR-0110 の未決定）。

## Scope

### 背景

`lint_and_fix()` は `FixResult.converged` / `conflicting_rules` / `conflict_cycle` を
破棄し、`rules_fixed` だけを `fixes_applied` に使う。未収束（振動や反復上限到達）でも
再 lint が通れば不安定な内容が確定し得る。

### 成果物

- `core/src/ai_markdown/lint.rs`: 変更（未収束情報を LintResultWrapper に追加）
- `core/src/ai_markdown/mod.rs`: 変更（未収束時の挙動、任意）
- `docs/decisions/0111-fix-convergence.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `LintResultWrapper` に `converged: bool` と `conflicting_rules: Vec<String>` を追加する。
2. `lint_and_fix()` で FixResult から値を設定する（`lint()` のみの経路は converged=true）。
3. 未収束時は fail-closed とする:
   - 振動（conflicting_rules が空でない）または反復上限到達で converged=false のとき、
     修正を採用せず元の content を `fixed_content` に戻し、
     `fixes_applied = 0`、`passed = false` とする。
4. テストを追加する。
5. 既存の 191 テスト、E2E 228 項目を壊さない。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- 新規依存を追加しない。
- 他モジュールの API 互換性を壊さない（フィールド追加は許容）。
- `master` へ push する。force push はしない。
