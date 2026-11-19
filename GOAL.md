# Goal

desktop / mobile の `build_app()` で重複している Tauri プラグイン設定を共通化する（R6 PR-L03）。

## Scope

### 背景

`desktop/src/main.rs` の desktop / mobile `build_app()` は
log / opener / fs / dialog / shell / store / notification / updater の
8 プラグイン設定が同一で、global_shortcut の有無だけが異なる。
プラグイン追加時に片方だけ更新されるリスクがある。

### 成果物

- `desktop/src/main.rs`: 変更（共通プラグイン設定関数の抽出）
- `docs/decisions/0201-build-app-plugin-dedup.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. 8 つの共通プラグインを 1 関数（例 `with_common_plugins()`）へ抽出する。
2. desktop はその関数に `global_shortcut` を追加し、mobile は追加しない。
3. プラグインの種類・設定・順序を維持する。
4. 全ゲート成功（E2E 251 維持）。

## Acceptance criteria

- 全ゲート成功、E2E 251 項目成功。

## Constraints

- Tauri プラグインの追加・削除・設定変更は行わない。
- 新規依存を追加しない。
- `master` へ push する。force push はしない。
