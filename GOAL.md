# Goal

PR-M08 / TD-M08 の設定モデル 3 重を「権威と寿命が異なるレイヤー分離」として設計受容する。

## Scope

### 背景

監査で、AppConfig / ReaderSettings / SettingsView は同じ値のコピーではなく、
core の永続設定 / web の localStorage UI 状態 / IPC 読み取り DTO という
異なる権威・寿命を持つことが確認された（WasmConfig は ADR-0205 で削除済み）。
統合は ADR-0018 / 0038 / 0157 / 0193 の契約と E2E 251 を壊すリスクが高い。

### 成果物

- `docs/decisions/0211-config-layer-boundaries.md`: 追加（設計受容）
- `todo.md`: PR-M08 を「設計受容」として完了へ移動
- `docs/decisions/index.md` / `docs/index.md` / `README.md` / `docs/log.md`: 更新
- `GOAL.md`: 本契約

### 必須事項

1. 3 モデルの責任分界を ADR に明記する。
2. localStorage キー `openrustbooks:settings` と camelCase スキーマを互換契約として固定する。
3. 統合を却下する根拠（wasm 単体の永続化、ADR-0157 契約、E2E、DTO 境界）を記載する。
4. 未決定（ReaderConfig 死設定、ConfigPatch 保存配線、auto_save_delay 供給元）を継続する。
5. markdownlint が通り、コード変更なし（E2E 251 維持）。

## Acceptance criteria

- markdownlint 0 issues、既存ゲート（E2E 251）を壊さない。

## Constraints

- コード・E2E は変更しない。
- 既存 ADR の本文は履歴として保持し、新規 ADR で design acceptance を記録する。
- `master` へ push する。force push はしない。
