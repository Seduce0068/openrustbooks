# Goal

REQ → ADR → テストの対応表と既存ソリューション選定根拠を整備する（R6 D02 / R4-trace#1,#4 / R5）。

## Scope

### 背景

`docs/e2e-traceability.md` は VC → E2E check（254 件）のみで、
REQ ID と ADR ID の列が無い。
要件の検証条件は「承認済みの受け入れ基準ではない」と明記されており、
測定可能な受け入れ基準への昇格も未決定だった。
選定根拠 ADR は散在し、ライセンス／鮮度／既知脆弱性を横断する
比較表が無い。

### 成果物

- `docs/e2e-traceability.md`: 変更（REQ / ADR 列と受け入れ基準の注記）
- `docs/decisions/0233-solution-selection-rationale.md`: 追加
- `docs/requirements.md`: 変更（VC を測定可能な受け入れ基準として参照）
- `todo.md`: D02 / R4-trace / R5 を完了へ移動
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. 各 VC に REQ ID と主要 ADR ID を対応付ける（E2E check 名は 254 件を維持）。
2. VC を「初版の測定可能な受け入れ基準」として参照する ADR を追加する。
3. 主要依存の選定根拠（ライセンス／鮮度／既知脆弱性／Wasm 対応）を表で記録する。
4. markdownlint 0 issues、verify.sh（E2E 254）成功。

## Acceptance criteria

- AST 254/254 一致、markdownlint 0 issues、verify.sh 成功。

## Constraints

- 既存 ADR の決定を上書きしない（参照と整理のみ）。
- E2E check 名・件数は変更しない。
- `master` へ push する。force push はしない。
