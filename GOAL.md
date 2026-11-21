# Goal

desktop fs コマンドで local_roots[].read_only を強制する（R6 レビュー / R3-AU01）。

## Scope

### 背景

create_file / delete_file / rename_file は create_storage() で local root を生成するが、
read_only フラグを参照しない。save_document だけが read_only を検査するため、
描画側の JS から fs コマンドを直接 invoke すると read-only root を書き換えられる。

### 成果物

- `desktop/src/commands/fs.rs`: 変更（read_only 判定と拒否）
- `desktop/src/state/mod.rs` または `core/src/io.rs`: 必要なら root 情報保持 API
- `docs/decisions/0285-fs-read-only-enforcement.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. create_file / delete_file / rename_file が対象 root の read_only を検査する。
2. read_only root への書き込み・削除・改名は固定文言で拒否する。
3. read_only=false の既存動作を壊さない。
4. 単体 test を追加する。
5. E2E 256 件・check 名・WDIO ゲートを維持し、verify.sh 全ゲート成功。

## Acceptance criteria

- read_only root への fs 書き込みが拒否され、通常 root は従来どおり動作する。

## Constraints

- 公開鍵 / endpoint / 外部サービスは扱わない。
- 秘密値をログへ出力しない。
- `master` へ push する。force push はしない。
