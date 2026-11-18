# Goal

About 画面のライセンスと対応形式の表示を実装事実に合わせる（ADR-0151 の次候補）。

## Scope

### 背景

`web/src/lib.rs:170-175` の About 画面は
`License: MIT`、`Supports: EPUB, PDF, Markdown, TXT, ZIP` と表示する。
実際は `Cargo.toml:12` が `AGPL-3.0-or-later` で、
README.md:81 は EPUB 閲覧のみ実装済み、他形式は設計段階としている。

### 成果物

- `web/src/lib.rs`: 変更（About 文言）
- `tools/verify-web-reader.py`: 変更（E2E チェック 1 件追加、231 → 232）
- `docs/decisions/0153-about-page-accuracy.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. ライセンス表示を `AGPL-3.0-or-later` に修正する。
2. 対応形式は「EPUB 閲覧は実装済み、TXT / Markdown / PDF / ZIP は設計段階」と正確に表示する。
3. E2E に About 画面へ遷移して表示を確認するチェックを 1 件追加し、期待総数を 232 に更新する。
4. 既存の E2E 231 項目と全ゲートを壊さない。

## Acceptance criteria

- E2E 232 項目が成功する。
- 6 ゲート + audit + wasm-test が成功する。

## Constraints

- コードの機能は変更せず、表示文言のみ。
- 新規依存を追加しない。
- `master` へ push する。force push はしない。
