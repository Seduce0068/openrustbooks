# Goal

部分設定インポートの bool false リセット懸念を調査し、no-op と判断する（ADR-0158）。

## Scope

### 背景

当初は `dark` / `followSystem` / `downscaleImages` の
`unwrap_or(false)` を `load_settings()` フォールバックへ変更し、
E2E を 234 → 235 にする予定だった。
しかし ADR-0157 の現在値保護ガードと `load_settings()` が同じ
`SETTINGS_KEY` を読むため、问题的な状態は自然には成立しないことが判明した。

### 成果物

- `docs/decisions/0158-partial-settings-import.md`: 追加（調査結果）
- `GOAL.md`: 最終契約へ更新
- コード変更なし（`web/src/lib.rs` / `tools/verify-web-reader.py` は HEAD と同一）

### 決定

1. 3 bool の `unwrap_or(false)` は現行挙動を維持する。
2. E2E 期待総数は 234 のまま。
3. ADR-0157 の「現在設定キーがある場合はスキップ」ガードを維持する。
4. 既定値を永続化する設計論点は ADR-0158 の未決定として記録する。

## Acceptance criteria

- markdownlint が 0 issues。
- 既存の全ゲート・E2E 234 項目が維持される。

## Constraints

- 新規依存を追加しない。
- `todo.md` と `docs/investigation/` は触らない。
- `master` へ push する。force push はしない。
