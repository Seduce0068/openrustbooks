# Goal

storage root 自身の破壊的操作と move_item の上書き競合を fail-closed 化する（R6 レビュー / R2-ER01 + R3-AU01）。

## Scope

### 背景

- `resolve_path('.')` は canonical root を返すため、
  `delete_file(path='.', recursive=true)` が root 配下全体を削除できる。
- `move_item` は overwrite=false でも POSIX `rename` が既存宛先を暗黙上書きし、
  overwrite=true は「宛先削除 → rename」で途中失敗時に元と宛先の両方を失う。

### 成果物

- `core/src/io.rs`: 変更（root 操作拒否、atomic replace / 事前再確認）
- `desktop/src/commands/fs.rs`: 必要ならエラー形状の確認
- `docs/decisions/0286-storage-destructive-guard.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. canonical target が storage root 自身なら delete / move_item / create_dir を拒否する。
2. move_item の overwrite=false は宛先が存在する場合に固定文言で拒否する。
3. overwrite=true は途中失敗でも元データを失わない方式にする。
4. read_only / containment / emit の既存動作を壊さない。
5. 単体 test を追加し、verify.sh 全ゲート成功。

## Acceptance criteria

- root 自身の削除・改名が拒否され、overwrite=false が既存宛先を保持する。

## Constraints

- 外部サービス / 秘密情報 / 公開設定は扱わない。
- E2E 256 件・check 名・WDIO ゲートを維持する。
- `master` へ push する。force push はしない。

## 完了記録

- storage root 自身の delete/move を固定文言で拒否。
- overwrite=false は既存宛先を保持、overwrite=true は退避→rename→削除/復元へ変更。
- create_dir(root) は root 直下作成を壊さないため idempotent 成功（R4 反映）。
- core 302 / desktop 107 tests、verify.sh 全ゲート成功（E2E 256、WDIO 1）。
- 原子的 no-replace rename と既存 io error のパス除去は ADR-0286 の未決定へ集約。
