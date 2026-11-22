# Goal

TableAlignment の未知値を fail-closed 化する（第2cycle R6 / R1-DS01）。

## Scope

### 背景

parse_table_format は未知値を CONFIG_VALIDATION で拒否する一方、
parse_table_alignment は未知値を Left へ黙ってフォールバックする。
クライアントの誤記が要求と異なる表を生成しても通知されない。

### 成果物

- `desktop/src/commands/ai.rs`: 変更
- `docs/decisions/0324-table-alignment-validation.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. parse_table_alignment を Result 化し、未知値を fixed code の validation エラーで拒否する。
2. None は従来どおり None を返す。
3. 大文字小文字無視・既存 left/center/right の成功契約を維持。
4. 単体 test を更新し、未知値拒否を固定する。
5. verify.sh 全ゲート成功、E2E 256 件・WDIO 維持。

## Constraints

- 外部 AI API への送信 test は行わない。
- `master` へ push する。force push はしない。

## 完了記録

- TableAlignment を Result 化し未知値を固定 validation で拒否、confirm 前に検証。
- wasm-bindgen-test-runner の PATH を ci-wasm-test.sh で解決。
- desktop 167 tests、verify.sh 全ゲート成功（E2E 256、WDIO 1）。
- 型レベル Deserialize 検証は ADR-0324 の未決定。

- web の status_message に CONFIG_LOAD_FAILED の固定日本語文言を追加。
- 保存系 code 5 種の専用文言と未知 code フォールバックを test で固定。
- web 20 tests、verify.sh 全ゲート成功（E2E 256、WDIO 1）。
- code 集合の共有定数同期は ADR-0325 の未決定。

## 完了記録（続き）

- README の verify.sh 前提 / runbook の --locked 表記 / url 脆弱性を更新。
- wasm-bindgen CLI / test runner を Cargo.lock 由来で版照合、負向 27 ケース detecting。
- verify.sh 全ゲート成功（E2E 256、WDIO 1）。
- 抽出関数の共通ファイル化は ADR-0326 の未決定。

## 完了記録（続き2）

- WDIO trunk serve に --locked --offline を追加、lock 不変を 4 時点で検査。
- runbook-webdriver を同期、verify.sh 全ゲート成功（E2E 256、WDIO 1）。
- 失敗経路の lock 可視化は ADR-0327 の未決定。
