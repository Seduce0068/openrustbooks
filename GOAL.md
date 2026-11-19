# Goal

ADR-0003 に rumdl feature の現状注記を追加する（R6 R2-crates#2）。

## Scope

### 背景

ADR-0003 は rumdl の default / wasm feature をターゲット別に選択すると
記録しているが、現在は native / wasm ともに featureless で、
wasm feature は同名 `init` export 衝突のため禁止されている（ADR-0160）。
後続 ADR-0155 / 0160 / 0168 / 0195 が上書き済みで、
ADR-0168:45 が「ADR-0003 本文修正」を未決定として残していた。

### 成果物

- `docs/decisions/0003-native-wasm-target-split.md`: 変更（現状注記を追加）
- `docs/decisions/0242-rumdl-feature-status.md`: 追加
- `todo.md`: R2-crates#2 を完了へ移動
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. ADR-0003 の 25 / 40-42 / 76 行目を「当時の背景」と明示し、
   現在は featureless であることを注記する。
2. 既存本文は履歴として保持し、削除しない。
3. ADR-0242 に上書き経緯と現状ゲート（`check-rumdl-featureless.sh`）を記録する。
4. workspace 依存の一元化・Wasm API 非公開化は未決定として残す。
5. markdownlint 0 issues、verify.sh（E2E 254）成功。

## Acceptance criteria

- 注記と索引が整合し、全ゲート成功。

## Constraints

- Cargo.toml / Cargo.lock / コード / ゲートスクリプトは変更しない。
- `master` へ push する。force push はしない。
