# Goal

Reader アンマウント時に Blob URL を解放する（R6 TD-M09）。

## Scope

### 背景

`object_urls` / `cover_url_state` は章再描画時と別ファイル読込時に
`revoke_object_url()` されるが、Reader がアンマウントされる
`on_cleanup`（web/src/lib.rs:3605）では解放していない。
SPA でルートを往復するたびに画像 Blob がブラウザメモリに残る。

### 成果物

- `web/src/lib.rs`: 変更（on_cleanup での URL 解放）
- `tools/verify-web-reader.py`: 変更（必要なら E2E 1 件、249 → 250）
- `docs/decisions/0192-blob-url-cleanup.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. Reader の `on_cleanup` で `object_urls` の全 URL と `cover_url_state` を
   `URL.revokeObjectURL()` する。
2. `on_cleanup` は `FnOnce() + Send + Sync + 'static` 要件があるため、
   `SendWrapper` で包んだ Rc をキャプチャする既存パターンを使う。
3. 章再描画・ファイル切替時の既存 revoke は維持する。
4. E2E 249 項目を壊さない（必要なら 1 件追加）。
5. 全ゲート成功。

## Acceptance criteria

- 全ゲート成功、E2E 249 項目（必要なら 250）成功。

## Constraints

- 新規依存を追加しない。
- 既存の画像表示・ダウンスケール挙動を変更しない。
- `master` へ push する。force push はしない。
