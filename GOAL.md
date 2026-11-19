# Goal

workspace 依存の第一段を統一した（R6 R2-crates#5、ADR-0246）。

## Scope

### 成果物

- `core/Cargo.toml`: feature 差分が無い 18 依存を workspace 参照化
- `docs/decisions/0246-workspace-dependency-unification.md`: 追加
- `todo.md`: 第一段完了、feature 加算依存は未決定
- README / 各索引 / log: 更新

### 決定

- rumdl / chrono / uuid / tokio wasm dev は除外。
- Cargo.lock 差分なし、check-rumdl-featureless 2 行維持。

## Acceptance criteria

- E2E 255 項目、全ゲート成功（検証済み）。
