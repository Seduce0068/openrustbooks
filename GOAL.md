# Goal

WasmMarkdownLinter を実 lint に接続し、fail-closed スタブを解消する（ADR-0156 の未決定）。

## Scope

### 背景

ADR-0156 で Wasm AI/lint を fail-closed に変更した。
lint は featureless rumdl が wasm ビルド可能で、native の
`MarkdownLinter` も wasm でコンパイルされるため、実装可能なら
ブラウザでも lint が動くようにできる（ADR-0110 / 0123 / 0156 の未決定）。

### 成果物

- `core/src/wasm.rs`: 変更（WasmMarkdownLinter の実装）
- `docs/decisions/0160-real-wasm-lint.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. 既存 `core/src/ai_markdown/lint.rs` の `MarkdownLinter` を wasm から利用し、
   `lint()` / `lint_and_fix()` が実際の結果を返すようにする。
2. `LintIssue` → `WasmLintIssue` の変換を追加する（end_line / end_column /
   severity / fixable を含む既存型に合わせる）。
3. 設定は `LintConfig::default()` または JsValue からの fail-closed 変換を使う。
   未知値・不正値は既定またはエラーにする（方針を ADR に記録）。
4. 収束情報（converged / conflicting_rules）の扱いを決める
   （WasmLintFixResult に追加する or (info) として渡す or omit）。
5. wasm32 check、wasm-pack test、native 全テスト、E2E 235 項目を壊さない。

## Acceptance criteria

- 全ゲート成功、E2E 235 項目成功。
- ADR に設計・制約が記録される。

## Constraints

- 新規依存を追加しない。
- rumdl の `wasm` feature は有効化しない（同名 export `init` 衝突）。
- AI 送信は本次対象外（ADR-0156 の AI fail-closed は維持）。
- `master` へ push する。force push はしない。
