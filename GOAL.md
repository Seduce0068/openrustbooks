# Goal

EPUB manifest item の欠落エラーを統一する（R6 R2-epub#3）。

## Scope

### 背景

`push_manifest_item()` は `id` 欠落時に `return Ok(())` で黙ってスキップしていたが、
`href` 欠落は `CoreError::archive()` で拒否していた。
ADR-0005 / 0059 / 0240 は不正な OPF を黙って読み飛ばさない方針を定めており、
`id` 欠落も fail-closed で報告するのが整合する。

### 成果物

- `core/src/epub.rs`: 変更（`id` 欠落を固定文言の `CoreError::archive` へ、test 追加）
- `docs/decisions/0248-epub-manifest-errors.md`: 追加
- `todo.md`: R2-epub#3 完了、#2 / #4 は残存
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. `id` 欠落時に `CoreError::archive("manifest の item に id がありません")` を返す。
2. エラー文言へ id 値・href 値・ファイル名を埋め込まない（ADR-0212 の redaction 契約を强化）。
3. `href` 欠落エラーからも `{id}` を削除し、固定文言へ統一する。
4. 既存 sample EPUB / E2E fixture はすべて id 付きで、壊れないことを確認する。
5. 全ゲート成功（E2E 255 維持）。

## Acceptance criteria

- core test 増加、E2E 255 項目、全ゲート成功。

## Constraints

- E2E check 名・件数は変更しない（unit test で十分）。
- `id=""` の扱いは未決定として残す（本次は属性欠落のみエラー化）。
- `master` へ push する。force push はしない。
