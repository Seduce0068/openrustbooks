# Goal

StorageBackend の TOCTOU と watch stub を整理する（R6 レビュー / R1-AR01 + R2-BV01）。

## Scope

### 背景

StorageBackend::move_item は overwrite の事前チェックと rename の間に
TOCTOU 窓がある。既存 ADR-0286 で再確認方式が上限と記録済み。
watch は未実装 stub で trait に残っており、実装も削除もされていない。

### 成果物

- `core/src/io.rs`: 変更（必要なら TOCTOU の追加対策、watch stub の整理）
- `docs/decisions/0308-storage-backend-contract.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. overwrite=false の競合、- 既存データ保持を現状より強化できるなら実装する。
2. watch stub を trait から分離するか、未実装契約を明示する。
3. canonicalize / open の競合を fail-closed にする（可能なら）。
4. 既存 read_only / containment / atomic write / move 契約を壊さない。
5. E2E 256 件・check 名・verify.sh 全ゲート維持。

## Constraints

- unsafe FFI / 新規依存は使わない。
- 新機能としての watch 実装は今回の範囲外。
- `master` へ push する。force push はしない。

## 完了記録

- write(overwrite=false) を create_new/hard_link 化、resolve_path 再確認を追加。
- watch 未実装契約を明記、退避先後始末を remove_path へ統一。
- core 346 / desktop 128 / web 9 tests、verify.sh 全ゲート成功（E2E 256、WDIO 1）。
- openat2 / hard_link 非対応 FS / watch trait 分離は ADR-0308 の未決定。
