# Goal

rumdl の default-features=false をローカルと CI の両方で静的検査する（R6 TD-M06、ADR-0195）。

## Scope

### 成果物

- `scripts/check-rumdl-featureless.sh`: 独立検査スクリプト（引数対応）
- `scripts/verify.sh`: 薄いラッパーへ置換
- `.github/workflows/ci.yml`: extra_gates に追加
- `docs/decisions/0195-rumdl-feature-gate.md`: 追加
- `todo.md`: TD-M06 を完了へ移動

### 決定

- 単一情報源のスクリプトを verify.sh と CI から呼ぶ。
- 共通ワークフロー自体は変更しない。
- 期待値固定（version 0.3 / 2 行）、不一致は fail-closed。

## Acceptance criteria

- E2E 250 項目、全ゲート成功（検証済み）。
