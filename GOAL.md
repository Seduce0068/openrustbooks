# Goal

read_directory を全 root 対応にする（第2cycle R6 / R1-DS01）。

## Scope

### 背景

read_directory は storages.first() 固定で一覧取得する。
複数 local_roots 構成で 2 番目以降の root を絶対パス指定すると
"Path escapes storage root" で拒否され、相対パスは先頭 root に解決される。
書き込み系は resolve_target_storage で最深一致 root を選択している。

### 成果物

- `desktop/src/commands/fs.rs`: 変更（read 専用 root 解決）
- `docs/decisions/0321-read-directory-root-resolution.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. read_directory が read_only を拒否しない read 専用解決ヘルパーを使う。
2. 絶対パス / 相対パス / symlink 経由で最深一致 root を選択する。
3. 複数 root の一覧取得の回帰 test を追加する。
4. E2E 256 件・check 名・read_only 読み取り許可を維持。
5. verify.sh 全ゲート成功。

## Constraints

- 外部サービス / 秘密情報は扱わない。
- `master` へ push する。force push はしない。
