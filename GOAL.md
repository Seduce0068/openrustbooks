# Goal

atomic write の一時ファイル名を一意化する（R6 PR-M06）。

## Scope

### 背景

`LocalStorage::write()` の atomic 分岐は
`full_path.with_extension("tmp")` という固定名を使う。
同一ファイルへの並行書き込みや既存の `.tmp` ファイルと衝突し、
片方のデータが他方の rename で上書きされる可能性がある。

### 成果物

- `core/src/io.rs`: 変更（一意な tmp 名、unit test）
- `docs/decisions/0188-atomic-tmp-uniqueness.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. tmp 名に PID とプロセス内カウンタ（AtomicUsize）を含める。
2. 既存の `.tmp` ファイルを上書きしない（create_new 相当の安全性）。
3. rename 成功後、tmp が残らないこと。
4. unit test を追加（並行 2 書き込みで衝突しない、tmp 残存なし）。
5. 全ゲート成功（E2E 249 維持）。

## Acceptance criteria

- 全ゲート成功、E2E 249 項目成功。

## Constraints

- 新規依存を追加しない。
- overwrite ガード（ADR-0173）は維持する。
- `master` へ push する。force push はしない。
