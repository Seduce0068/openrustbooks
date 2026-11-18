# Goal

エディタの Save をブラウザ内 localStorage ドラフトとして接続する（ADR-0161 の未決定）。

## Scope

### 背景

ADR-0161 で textarea と Preview を接続したが、Save は
「desktop only / not implemented」の恒久 disabled。
Tauri invoke は有効化しない方針（ADR-0128）を維持しつつ、
ブラウザ単体でも明示保存によるドラフト保持を実現する。

### 成果物

- `web/src/lib.rs`: 変更（Save 有効化、ドラフト読み書き、status 表示）
- `tools/verify-web-reader.py`: 変更（E2E 2 件追加、238 → 240）
- `docs/decisions/0164-editor-draft-save.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. キー `openrustbooks:editor:{doc_id}` に本文を保存する。
2. Save ボタンを有効化し、クリック時に現在の `content` を保存し、
   status に「下書きを保存しました」を表示する。失敗時は ADR-0154 の
   保存失敗通知を再利用する。
3. エディタ表示時に同じキーのドラフトがあれば textarea / content へ復元する。
4. 保存は明示操作のみ（自動保存しない）。既存の読書位置／設定キーは触らない。
5. E2E に 2 件追加:
   - 入力→Save→localStorage に保存→リロード後に復元。
   - 保存失敗通知が出ること（ADR-0154 の経路を再利用可）。
6. 既存 238 項目と全ゲートを壊さない。期待総数 238 → 240。

## Acceptance criteria

- E2E 240 項目が成功する。
- 6 ゲート + audit + wasm-test が成功する。

## Constraints

- Tauri invoke は有効化しない。
- 新規依存を追加しない。
- 機微情報（本文）を status / console に出さない。
- `master` へ push する。force push はしない。
