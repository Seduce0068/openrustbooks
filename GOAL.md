# Goal

部分設定のインポートで bool が false にリセットされる問題を修正する（ADR-0157 の未決定）。

## Scope

### 背景

`web/src/lib.rs` の `import_positions()` は設定の
`dark` / `followSystem` / `downscaleImages` を `unwrap_or(false)` で読む。
バックアップに这些キーが欠ける場合、現在設定の true が false へ
リセットされる（ADR-0151 / ADR-0157 の未決定）。

### 成果物

- `web/src/lib.rs`: 変更（`unwrap_or_else(|| load_settings().xxx)` に統一）
- `tools/verify-web-reader.py`: 変更（E2E 1 件追加、234 → 235）
- `docs/decisions/0158-partial-settings-import.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. 3 つの bool フィールドについて、キー欠落時は
   `load_settings()` の現在値をフォールバックにする。
2. 5 つの数値フィールドは既に現在値フォールバックのため維持。
3. E2E に 1 件追加（現在設定 dark=true / followSystem=true /
   downscaleImages=true の状態で、それら 3 キーを除くバックアップを
   新規環境へインポートしても true が維持される）。
   期待総数 234 → 235。
4. 既存 234 項目と全ゲートを壊さない。

## Acceptance criteria

- E2E 235 項目が成功する。
- 6 ゲート + audit + wasm-test が成功する。

## Constraints

- 新規依存を追加しない。
- 設定スキーマは変更しない。
- `master` へ push する。force push はしない。
