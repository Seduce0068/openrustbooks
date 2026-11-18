# Goal

エディタ下書きをバックアップのエクスポート／インポート対象に含める（R6 TD-H06）。

## Scope

### 背景

Web エディタの唯一の保存先は localStorage の `openrustbooks:editor:{doc_id}` だが、
`export_positions()` は positions / settings / history のみを出力し、
バックアップ復元時に本文が戻らない（R6 TD-H06）。

### 成果物

- `web/src/lib.rs`: 変更（export に drafts 追加、import で復元、成否返却）
- `tools/verify-web-reader.py`: 変更（E2E 1 件、248 → 249）
- `docs/decisions/0180-draft-backup.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. `EXPORT_VERSION` は **1 のまま**とし、`drafts` を optional field として追加する
   （旧形式のインポート互換を維持）。
2. export は `openrustbooks:editor:` で始まるキーを
   `{ "drafts": { key: content } }` として含める。
3. import は `drafts` が存在する場合、現在同じキーが無いときのみ復元する
   （ADR-0157 の現在値保護を維持）。
4. 容量上限（1 MiB）を超える保存済み下書きはバックアップへ含めない
   （ADR-0167 / 0170 の方針を維持）。
5. E2E 1 件追加（ラウンドトリップで下書きが復元される）。期待総数 248 → 249。
6. 全ゲート成功。

## Acceptance criteria

- E2E 249 項目、全ゲート成功。

## Constraints

- 新規依存を追加しない。
- 既存の positions / settings / history 契約を壊さない。
- 本文を status / console / エラー文言へ出さない。
- `master` へ push する。force push はしない。
