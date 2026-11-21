# Goal

ユーザー入力のサイズ境界を fail-closed 化する（R6 レビュー / R3-IV01）。

## Scope

### 背景

desktop `open_document` はファイルを無制限に `read_to_string` し、
web は `file.size()` 確認前に `array_buffer()` を呼ぶ箇所がある。
import JSON もサイズ無制限で、巨大入力によりメモリ消費／UI フリーズが起き得る。

### 成果物

- `desktop/src/commands/document.rs`: 変更
- `web/src/lib.rs`: 変更
- `docs/decisions/0301-input-size-bounds.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. desktop open は `DEFAULT_MAX_READ_BYTES` を超える前に拒否する。
2. web は `file.array_buffer()` の前に `file.size()` を検査する。
3. import JSON も共通上限で拒否する。
4. 既存 EPUB 上限 / 下書き上限を壊さない。
5. 単体 / WDIO test は既存 256 件・check 名を維持。
6. verify.sh 全ゲート成功。

## Constraints

- 上限は 256 MiB（core::io::DEFAULT_MAX_READ_BYTES）に合わせる。
- 秘密値 / 絶対パスをエラーへ出さない。
- `master` へ push する。force push はしない。
