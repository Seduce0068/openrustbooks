# Goal

Leptos の非公開 re-export ではなく send_wrapper を直接依存として使う（R6 TD-M07、ADR-0200）。

## Scope

### 成果物

- `web/Cargo.toml`: send_wrapper = "0.6" を直接依存化
- `web/src/lib.rs`: import を公開クレートパスへ
- `Cargo.lock`: web の dependencies に 1 行追加（新規クレートなし）
- `docs/decisions/0200-send-wrapper-direct-dep.md`: 追加
- `todo.md`: TD-M07 を完了へ移動

### 決定

- doc(hidden) re-export から脱却し、既存 transitive 0.6 を直接化。
- cleanup 挙動は不変、unsafe 追加なし。

## Acceptance criteria

- E2E 251 項目、全ゲート成功（検証済み）。
