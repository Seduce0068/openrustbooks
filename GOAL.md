# Goal

エディタ下書きをバックアップのエクスポート／インポート対象に含める（R6 TD-H06、ADR-0180）。

## Scope

### 成果物

- `web/src/lib.rs`: export に drafts 追加、import で復元、現在値保護
- `tools/verify-web-reader.py`: E2E 1 件、248 → 249
- `docs/decisions/0180-draft-backup.md`: 追加
- `todo.md`: TD-H06 を完了へ移動

### 決定

- EXPORT_VERSION=1 のまま optional `drafts` を追加（旧互換）。
- 1 MiB 超は export/import 双方で除外。
- 現在キーが存在する場合は上書きしない（ADR-0157）。
- 本文・キー名・サイズ値は status / console へ出さない。

## Acceptance criteria

- E2E 249 項目、全ゲート成功（検証済み）。
