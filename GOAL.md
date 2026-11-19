# Goal

バージョン情報の二重管理を解消した（R6 R3-release#3、ADR-0239）。

## Scope

### 成果物

- `desktop/tauri.conf.json`: version 削除
- `scripts/check-version-consistency.sh`: 新規静的ゲート
- `scripts/verify.sh` / `.github/workflows/ci.yml`: 配線
- `docs/decisions/0239-version-single-source.md`: 追加
- `todo.md` / README / 各索引 / log: 更新

### 決定

- Cargo workspace を単一情報源、Tauri 2 の version 省略 fallback を利用。
- version 再混入を fail-closed の静的ゲートで検出。
- build.rs 生成・TAURI_CONFIG 上書きは却下。

## Acceptance criteria

- 負向 test が失敗し、verify.sh 全ゲート成功（検証済み）。
