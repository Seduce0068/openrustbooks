# Goal

ローカル_filesystem のパス解決を fail-closed にする（todo.md 独自最優先 / code-scanner #1）。

## Scope

### 背景

`core/src/io.rs` の `LocalStorage::resolve_path()` は絶対パスを無条件に採用し、
`..` も正規化せずに root へ結合する。Tauri コマンド経由で任意パスの
読み書きが可能になり得る（UI 未接続のため現時点では潜在リスク）。

### 成果物

- `core/src/io.rs`: 変更（fail-closed なパス解決＋テスト）
- `docs/decisions/0097-path-containment.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. 絶対パス・`..` による root 脱出を拒否する。
2. 既存の相対パス（root 配下の通常のパス）は従来どおり動く。
3. シンボリックリンクで root 外へ出る場合もできれば拒否する。
4. 新規依存を追加しない。
5. core の単体テストを追加する。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- `desktop/` は変更しない（呼び出し側の修正は別タスク）。
- Web 版には影響しない。
- `master` へ push する。force push はしない。
