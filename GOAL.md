# Goal

バージョン情報の二重管理を解消し、drift を静的ゲートで防ぐ（R6 R3-release#3）。

## Scope

### 背景

workspace `Cargo.toml:10` と `desktop/tauri.conf.json:4` が
ともに `0.1.0` を保持していた。Tauri 2 は `version` キー省略時に
Cargo のバージョンを使う公式動作を持つため、JSON 側の固定値は不要。
削除だけでは将来の再混入を防げないため、静的ゲートも追加する。

### 成果物

- `desktop/tauri.conf.json`: 変更（`version` キー削除）
- `scripts/check-version-consistency.sh`: 追加
- `scripts/verify.sh`: 変更（`run version-consistency ...`）
- `.github/workflows/ci.yml`: 変更（`extra_gates` に 1 行）
- `docs/decisions/0239-version-single-source.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. `desktop/tauri.conf.json` から `version` を削除し、Cargo workspace を単一情報源にする。
2. 新規スクリプトで tauri.conf.json に `version` が無いことを fail-closed で検査する。
3. `verify.sh` と `ci.yml` の両方から同じスクリプトを呼ぶ。
4. 不正 JSON / `version` 再混入 / ファイル欠落の負向 test を実施する。
5. markdownlint 0 issues、verify.sh（E2E 254）成功。

## Acceptance criteria

- 負向 test が失敗し、通常 run が全ゲート成功。

## Constraints

- 共通ワークフロー `rust-gates.yml@v1` は変更しない。
- 新規依存を追加しない（`jq` が無い環境でも動く Python stdlib を使用可）。
- `master` へ push する。force push はしない。
