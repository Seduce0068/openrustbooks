# Goal

Markdown 文書の保存時に front matter が失われるバグを修正する（todo.md 独自 R2-arch-state#1）。

## Scope

### 背景

`desktop/src/commands/document.rs` の保存処理は `document.as_text()` を使うが、
`Document::as_text()` は Markdown の body のみを返すため、
`DocumentContent::Markdown { front_matter, .. }` がディスク書き込み時に失われる。

### 成果物

- `core/src/document.rs`: 変更（front matter を結合した完全テキストを返すメソッド）
- `desktop/src/commands/document.rs`: 変更（保存時に新メソッドを使う）
- `GOAL.md`: 本契約

### 必須事項

1. `Document` に `full_text()`（仮）を追加し、Markdown の場合は
   front matter を `---` で囲み body の前に結合して返す。
   - front matter が `None` の場合は body をそのまま返す。
   - `Text` の場合は従来どおり本文を返す。
2. 保存処理で `as_text()` の代わりに新メソッドを使う。
3. 既存の `as_text()` は編集用 API として維持する（挙動変更なし）。
4. core に単体テストを追加する。
5. 既存の 125 テスト、E2E 220 項目を変えない。

## Acceptance criteria

- 6 ゲートすべて成功する。
- 新しい単体テストが通る。

## Constraints

- 新規依存を追加しない（既存の `serde_norway` を使用）。
- Web 版には影響しない。
- `master` へ push する。force push はしない。
