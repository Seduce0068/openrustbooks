# Goal

設定リセットが保存失敗 status を成功メッセージで上書きする問題を修正する（R6 PR-H05）。

## Scope

### 背景

`on_reset_settings()` は `save_settings()` の成否を検査せず、
直後に `set_status("Display settings reset.")` を実行する。
保存失敗時に `report_save_failure()` が同期 dispatch した固定文言が上書きされる
（ADR-0154 / 0165 の通知が見えない）。

### 成果物

- `web/src/lib.rs`: 変更（save_settings の成否を返す／reset 側で検査）
- `tools/verify-web-reader.py`: 変更（E2E 1 件、246 → 247）
- `docs/decisions/0175-reset-settings-failure.md`: 追加
- `GOAL.md` / `todo.md`: 本契約・完了反映

### 必須事項

1. `save_settings()` が成功 bool を返すようにし、既存呼び出し側は戻り値を無視できる形を維持。
2. `on_reset_settings()` は失敗時に成功文言を出さない（失敗通知は ADR-0165 の経路を維持）。
3. E2E に 1 件追加（setItem を一時失敗させ Reset settings をクリック → 失敗 status / console 通知、成功文言が出ない）。
4. 期待総数 246 → 247。
5. 全ゲート成功。

## Acceptance criteria

- E2E 247 項目、全ゲート成功。

## Constraints

- 新規依存なし。
- 既存の成功文言 "Display settings reset." は維持。
- `master` へ push する。force push はしない。
