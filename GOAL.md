# Goal

要件の検証条件と E2E check 名の対応表を追加し、トレーサビリティを可視化する（R6 TD-L07）。

## Scope

### 背景

`docs/requirements.md:140-153` の検証条件は ID を持たず、`tools/verify-web-reader.py` の
251 件の check 名との対応も無い。ADR-0090 の総数ゲートだけでは、
要件から E2E 項目が消えても検出できない（TD-L05 の古いコメントは既に解消済み）。

### 成果物

- `docs/e2e-traceability.md`: 追加（VC ID → 要件 → E2E check 名）
- `docs/index.md` / `docs/log.md`: 更新
- `todo.md`: TD-L05 / TD-L07 を完了へ移動
- `GOAL.md`: 契約

### 必須事項

1. `docs/requirements.md` の検証条件と品質要件に VC-001... の ID を付与する（元の文言は維持）。
2. `docs/e2e-traceability.md` に VC ID → REQ → E2E check 名（実在）の表を記載する。
3. 未カバー要件（Drive / OS連携 / 5 ブラウザ等）は理由と未決定を明記する。
4. E2E check 名は `tools/verify-web-reader.py` から抽出した実名のみを使う。
5. markdownlint が通り、既存ゲート（E2E 251）を壊さない。

## Acceptance criteria

- markdownlint 0 issues、全ゲート（既存の緑）維持。

## Constraints

- E2E check 名・件数は変更しない。
- 新規依存を追加しない。
- `master` へ push する。force push はしない。
