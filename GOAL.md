# Goal

EPUB 非同期ロードに独立した世代カウンタを導入する（R6 PR-H03/TD-H03、ADR-0176）。

## Scope

### 成果物

- `web/src/lib.rs`: load_generation と 10 箇所のガード
- `docs/decisions/0176-epub-load-generation.md`: 追加
- `todo.md`: PR-H03/TD-H03 を完了へ移動

### 決定

- 描画用 current_generation とは別に load_generation（u64）を置く。
- インクリメントは検証通過後・key 計算前。
- 全 UI 書き込み前に世代照合。
- E2E は 247 維持（再現フックが無くフレークするため、ADR に理由記録）。

## Acceptance criteria

- E2E 247 項目、全ゲート成功（検証済み）。
