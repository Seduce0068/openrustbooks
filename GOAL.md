# Goal

save-error 通知のリスナーを共通化し、Editor / Reader の status 競合を解消する（ADR-0164 の未決定）。

## Scope

### 背景

ADR-0154 で追加した `openrustbooks:save-error` リスナーは Reader に 1 つだったが、
ADR-0164 で Editor にも追加した結果、両方が同じイベントを受け取り得る。
SPA 遷移時は排他と推測されるが、同時存在時に status が競合するリスクがある。

### 成果物

- `web/src/lib.rs`: 変更（共通リスナー／emit 先の明示）
- `tools/verify-web-reader.py`: 変更（必要なら E2E 1 件、240 → 241）
- `docs/decisions/0165-save-error-listener.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `report_save_failure()` が発行するイベントに、対象画面（reader / editor）を
   識別できる情報を含めるか、リスナーを 1 箇所へ集約する。
2. Editor / Reader の両方が同時に存在しても、誤った画面の status を上書きしない。
3. 既存の 3 経路（下書き / 読書位置 / 表示設定 / 履歴）の通知を壊さない。
4. E2E 240 項目（必要なら +1）と全ゲートを壊さない。

## Acceptance criteria

- 全ゲート成功、E2E 240 または 241 項目成功。

## Constraints

- 新規依存を追加しない。
- 本文・値をイベントへ載せない（ADR-0154 / 0164 の原則維持）。
- `master` へ push する。force push はしない。
