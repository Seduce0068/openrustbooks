# Goal

`AppConfig::save()` を tmp ファイル＋rename による原子書き込みへ変更する（R6 TD-L09、ADR-0177 / ADR-0207 の残リスク）。

## Scope

### 背景

`core/src/config.rs` の `write_private()` は固定パスを `truncate(true)` で開くため、
保存途中の失敗で config.toml が部分的に破壊される可能性がある。
ADR-0188 で `LocalStorage::write()` の tmp 名は一意化済みだが、
設定保存には及んでいない。

### 成果物

- `core/src/config.rs`: 変更（tmp + rename の原子保存、unit test）
- `docs/decisions/0209-atomic-config-save.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. `AppConfig::save()` が同じディレクトリに一意な tmp ファイルを作成し、
   書き込み完了後に `rename()` で config.toml へ置換する。
2. tmp 名にプロセス ID とカウンタを含め、既存 tmp を上書きしない。
3. Unix では tmp ファイルも 0600 パーミッションにする（既存 `write_private` 契約を維持）。
4. 書き込み／rename 失敗時に tmp を best-effort で削除する。
5. unit test を追加（保存内容、tmp 残存なし、既存 config.toml の置換）。
6. 全ゲート成功（E2E 251 維持）。

## Acceptance criteria

- 全ゲート成功、E2E 251 項目成功。

## Constraints

- 新規依存を追加しない。
- wasm32 ビルドを壊さない（native-only のまま）。
- 既存の設定保存先・半径（ProjectDirs）は変更しない。
- `master` へ push する。force push はしない。
