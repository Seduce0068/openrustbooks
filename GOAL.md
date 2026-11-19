# Goal

E2E 件数の current 表記ズレを自動検出する静的ゲートを追加する（ADR-0262 の未決定を解消）。

## Scope

### 背景

`Checker(expected=255)` が出典だが、
`todo.md` / `docs/documentation-operations.md` の current 値は手運用で追従していた。
ADR-0262 は自動検出ゲートの要否を未決定として残していたが、
version-consistency と同型の静的検査で fail-closed に検出できる。

### 成果物

- `scripts/check-e2e-count-consistency.sh`: 追加
- `scripts/verify.sh`: 変更（静的ゲート追加）
- `.github/workflows/ci.yml`: 変更（extra_gates に 1 行）
- `docs/decisions/0263-e2e-count-gate.md`: 追加
- `docs/documentation-operations.md`: 変更（ゲート運用を明記）
- `todo.md`: CI-H01 の未決定を解消（billing は残す）
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. Python AST とテキスト走査で `Checker(expected=N)` と
   current 表記（todo CI-H01 / documentation-operations）の一致を検査する。
2. 期待値・root を引数で差し替え可能にし、負向 test（不一致・欠落・不正形式）で失敗する。
3. `verify.sh` と `ci.yml` の両方から同じ script を呼ぶ。共通 workflow は変更しない。
4. E2E check 名・件数・トレーサビリティは変更しない（255 維持）。
5. 全ゲート成功。

## Acceptance criteria

- 負向 test が失敗し、通常 run で全ゲート成功（E2E 255）。

## Constraints

- `tools/verify-web-reader.py` / `docs/e2e-traceability.md` / Cargo.* は変更しない。
- 外部サービス・billing は扱わない。
- `master` へ push する。force push はしない。

## 完了記録

- ADR-0262 の未決定を解消する e2e-count-consistency ゲートを追加。
- AST で Checker(expected=N) と check/equal 内訳・重複を検査、current 2 アンカーを固定。
- 負向 test 15/15 PASS、`./scripts/verify.sh` 全ゲート成功、E2E 255 passed / 0 failed。
