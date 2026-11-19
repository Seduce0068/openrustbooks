# Goal

ZIP エントリの実展開バイトにセッション予算を設ける（R6 R1-archive#2）。

## Scope

### 背景

`ArchiveLimits::max_total_size` はセントラルディレクトリの宣言サイズ合計のみを検査し、
`read_entry()` は1回あたり `max_entry_size` で打ち切る。
しかし `&self` の `read_entry()` を繰り返すと、
宣言合計 256 MiB を超える実展開を累積できた（wasm の JS 公開 API を含む）。

### 成果物

- `core/src/archive.rs`: 変更（非公開 `AtomicU64` 予算、予約／返金、unit test）
- `docs/decisions/0251-expanded-size-budget.md`: 追加
- `todo.md`: R1-archive#2 完了、#3 残存
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. `ArchiveReader` に非公開の `AtomicU64` を追加し、
   `read_entry_with_limit()` の実展開バイト累計を `limits.max_total_size` で制限する。
2. 公開 API（`&self` / `Send + Sync` / `ArchiveLimits` のフィールド）は変更しない。
3. 予約は上限超過時に失敗し、読み出し失敗時は予約を返金する。
4. 実バイトが予約より少ない場合は差額を返金する。
5. テスト: 再読拒否、失敗時返金、上限内成功、`Send + Sync` 回帰。
6. 全ゲート成功（E2E 255 維持）。

## Acceptance criteria

- core test 増加、E2E 255 項目、全ゲート成功。

## Constraints

- `&mut self` 化・`Mutex` 依存・新規 public field は禁止。
- エラー文言にアーカイブ内パスや生エラーを出さない（固定文言を優先）。
- `master` へ push する。force push はしない。
