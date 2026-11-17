# Goal

配布 WASM から開発ホストの絶対パスを除去する（todo.md E01）。

## Scope

### 背景

配布用 WASM（`web/dist/*_bg.wasm`）に `/home/koppe/...` などの開発ホスト
絶対パスが 1811 件含まれている。デバッグ情報やパニックメッセージから
開発環境のパスが漏れる可能性がある。

### 成果物

- `Cargo.toml`: 変更（ワークスペース共通の `remap-path-prefix` 設定）
- `tools/verify-web-reader.py`: 変更（配布 WASM のパス漏れ検証を追加、任意）
- `docs/decisions/0088-wasm-path-stripping.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. ワークスペースの `[profile.release]` に `-C remap-path-prefix` を設定し、
   ビルド時の絶対パスを相対パスへ置換する。
2. `trunk build --offline` で生成した WASM に開発ホスト絶対パス
   （`/home/koppe`、`/home/`、`/tmp/`）が含まれないことを確認する。
3. 既存の 126 テスト、E2E 220 項目を変えない。

## Acceptance criteria

- 配布 WASM に開発ホスト絶対パスが 0 件。
- 6 ゲート・E2E が成功する。

## Constraints

- 新規依存を追加しない。
- 既存の `strip = true` は維持する。
- `desktop/` の Tauri 設定は変更しない。
- `master` へ push する。force push はしない。
