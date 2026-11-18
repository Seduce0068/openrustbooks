# Goal

auto_fix 分岐の native / wasm 重複を共通関数へ集約する（R6 PR-M09、ADR-0194）。

## Scope

### 成果物

- `core/src/ai_markdown/lint.rs`: validate_with_auto_fix + 直接テスト 2 件
- `core/src/ai_markdown/mod.rs` / `core/src/wasm.rs`: 共通関数へ委譲
- `docs/decisions/0194-lint-validation-unification.md`: 追加
- `todo.md`: PR-M09 を完了へ移動

### 決定

- 共通関数は CoreResult を返し、JsValue 化は境界側。
- LintResultWrapper の形・意味は不変。
- wasm の map_err(js_error) と omit 契約を維持。

## Acceptance criteria

- core 248 / E2E 250、全ゲート成功（検証済み）。
