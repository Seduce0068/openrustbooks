# Goal

`StorageBackend::read()` の offset / limit に上限を適用する（R6 PR-M04）。

## Scope

### 背景

`ReadOptions.limit` は `u64` のままで `file.take(limit)` に渡され、
`StorageCapabilities.max_file_size` と関連付けられていない。
巨大な limit を渡すとファイル全体を Vec に読み込み、
レスポンス直列化も含めてメモリを圧迫し得る。

### 成果物

- `core/src/io.rs`: 変更（既定上限、clamp／拒否、unit test）
- `docs/decisions/0190-read-limit-hardening.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. `pub const DEFAULT_MAX_READ_BYTES: u64 = 256 * 1024 * 1024;` を追加する。
2. `LocalStorage::read()` で `options.limit` を
   `self.capabilities().max_file_size.unwrap_or(DEFAULT_MAX_READ_BYTES)` 以下へ
   clamp する（または上限超を明示エラーにする）。
3. `limit=None` でも上限を適用する（全読もうとしても無制限にならない）。
4. offset がファイル末尾を超える場合は空 Vec を返す（既存の seek 挙動を維持）。
5. unit test を追加（limit の clamp、None の既定適用、offset 超過）。
6. 全ゲート成功（E2E 249 維持）。

## Acceptance criteria

- 全ゲート成功、E2E 249 項目成功。

## Constraints

- 新規依存を追加しない。
- 既存の `read_text()` / `write()` 契約を壊さない。
- `master` へ push する。force push はしない。
