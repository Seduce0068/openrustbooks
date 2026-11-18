# Goal

上限超えの保存済み下書きを復元する際に固定警告を表示する（ADR-0167 の未決定）。

## Scope

### 背景

ADR-0167 で新規保存は 1 MiB に制限したが、`load_editor_draft()` は
保存済みの 1 MiB 超本文も無警告で復元する。
復元側で削除／拒否すると利用者のデータを失うため、表示のみで安全に通知する。

### 成果物

- `web/src/lib.rs`: 変更（復元時のサイズ確認と固定警告）
- `tools/verify-web-reader.py`: 変更（E2E 1 件、243 → 244）
- `docs/decisions/0170-oversized-draft-restore.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. 保存済み下書きが `EDITOR_DRAFT_MAX_BYTES` を超えていても、
   本文は復元する（データ損失禁止）。
2. 復元時に上限超えを検出したら、`.editor-status` に固定文言
   「保存済みの下書きが大きすぎます。編集して保存してください。」を表示する。
3. 本文・サイズ値は status / console / イベントへ載せない。
4. E2E に 1 件追加（上限超の下書きを直接 localStorage へ置き、
   /editor/e2e を開くと本文が復元され警告が出る）。期待総数 243 → 244。
5. 既存 243 項目と全ゲートを壊さない。

## Acceptance criteria

- E2E 244 項目が成功する。
- 6 ゲート + audit + wasm-test が成功する。

## Constraints

- 過剰な下書きを自動削除・切断しない。
- 新規依存を追加しない。
- `master` へ push する。force push はしない。
