# Goal

正規化後同名の ZIP エントリを fail-closed で拒否する（R6 R1-archive#4）。

## Scope

### 背景

`ArchiveReader` は `index.entry(path).or_insert(position)` で
正規化後同名のエントリを先着保持していた。
core 内では整合するが、`entries()` が重複を露出するため、
外部の consumer が後勝ちを選ぶと parser differential が生じ得る。
ADR-0004 の「逸脱エントリはアーカイブ全体を拒否」方針と整合させる。

### 成果物

- `core/src/archive.rs`: 変更（`insert` + 重複検出、unit test）
- `core/src/wasm.rs`: 変更（wasm test に重複ケース）
- `docs/decisions/0240-duplicate-archive-entries.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. `index.insert(path.clone(), position).is_some()` なら
   `CoreError::Validation` を返す。
2. 完全相同、`./` 正規化、二重スラッシュ、バックスラッシュ、
   `a/./b` の重複を検証する。
3. `mimetype` 重複が `EpubBook::open()` でも拒否されることを確認する。
4. 正規化後に別名のエントリは従来どおり開けることを維持する。
5. 全ゲート成功（E2E 254 維持）。

## Acceptance criteria

- core / wasm test が増加し、全ゲート成功。

## Constraints

- `ArchiveEntry` / 公開 API / `Send + Sync` は変更しない。
- 累計実展開上限と明示的 XML 深さ上限は ADR の未決定へ残す。
- `master` へ push する。force push はしない。
