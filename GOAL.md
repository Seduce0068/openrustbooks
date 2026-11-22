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

## 完了記録（続き3）

- reader.font_size / line_height を初回のみ localStorage 初期値化。
- 既存設定を守り、権威 / 未配線 / 失敗時の固定文言を統一。
- web 26 tests、verify.sh 全ゲート成功（E2E 256、WDIO 1）。
- editor 配線 / 即時 signal 更新は ADR-0328 の未決定。

## 完了記録（続き4）

- EPUB2 NCX サンプルと 2 check を追加、Checker(expected)=258。
- e2e-traceability / documentation-operations も 258 へ更新。
- verify.sh 全ゲート成功（E2E 258、WDIO 1）。
- VC 行の分割は ADR-0329 の未決定。

## 完了記録（続き5）

- prefers-contrast: more を CDP emulation + WCAG 比で検証（5 check）。
- Checker(expected)=263、E2E/traceability/documentation-operations を更新。
- verify.sh 全ゲート成功（E2E 263、WDIO 1）。
- 既存リテラル比較の比ベース化は ADR-0330 の未決定。

## 完了記録（続き6）

- get_config 失敗を desktop only / IPC 失敗で固定文言分岐。
- WDIO reject 注入は tauri-service 競合で却下、単体 test で代替。
- web 28 tests、verify.sh 全ゲート成功（E2E 263、WDIO 1）。
- 再読み込みボタンは ADR-0331 の未決定。

## 完了記録（続き7）

- capability ゲートの source 検査範囲と実 WebView 到達性を明示。
- verify.sh 全ゲート成功（E2E 263、WDIO 1）。
- ACL / withGlobalTauri の見直しは ADR-0332 の未決定。

## 完了記録（第3cycle-1）

- 配布保持ゲートを Checker(expected) AST 抽出へ接続。
- self_test ログを stderr 化し --dist 保持フローを修正。
- verify.sh --dist 実実行で保持成功、E2E 263 / WDIO gate rc=0。
- verify.sh 側 grep 抽出の共通化は ADR-0333 の未決定。

## 完了記録（第3cycle-2）

- set_document_content を 3 ロック同時保持で原子化。
- close 競合 test / last-write-wins を明記。
- desktop 176 tests、verify.sh 全ゲート成功（E2E 263、WDIO 1）。
- 本文マージは ADR-0334 の未決定。

## 完了記録（第3cycle-3）

- lint / auto-fix を LintConfig 直接構築へ変更、既定 AI 無効で動作。
- 新規 4 test、desktop 180 passed。
- verify.sh 全ゲート成功（E2E 263、WDIO 1）。
- 未使用 AppHandle の扱いは ADR-0335 の未決定。

## 完了記録（第3cycle-4）

- API キー空／空白入力を no-op 化し設定済みキーを保護。
- web 31 tests、verify.sh 全ゲート成功（E2E 263、WDIO 1）。
- 削除確認ダイアログ / command 未呼び出し E2E は ADR-0336 の未決定。

## 完了記録（第3cycle-5）

- save 競合時にディスクを元内容へ復元、復元失敗は別固定文言。
- 退避 read 前後のサイズ検査と失敗注入 3 経路を追加。
- desktop 184 tests、verify.sh 全ゲート成功（E2E 263、WDIO 1）。
- 退避 read 本体の注入 / 明示 drop は ADR-0337 の未決定。

## 完了記録（第3cycle-6）

- AI 初期化エラーのログを code() 由来へ変更。
- verify.sh 全ゲート成功（E2E 263、WDIO 1）。
- UI status 伝播は ADR-0338 の未決定。

## 完了記録（第3cycle-7）

- delete 直前の resolve 再確認を追加、ADR-0286 方針を適用。
- verify.sh 全ゲート成功（E2E 263、WDIO 1）。

## 完了記録（第3cycle-8）

- AI 設定差分のみ service 再生成、lint のみ変更は現在 service を維持。
- 旧 helper を test module 内へ移し clippy 修正。
- verify.sh 全ゲート成功（E2E 263、WDIO 1）。

## 完了記録（第3cycle-9）

- CONFIG_CONFLICT 用の再読み込みボタンを追加。
- verify.sh 全ゲート成功（E2E 263、WDIO 1）。
- 未保存編集の扱いは ADR-0341 の未決定。

## 完了記録（第3cycle-10）

- prefer_local を DTO / UI から除外し config.toml 値を保持。
- DTO contract gate に両側除外を追加、575 tests pass。
- verify.sh 全ゲート成功（E2E 263、WDIO 1）。
- provider 選択への実配線は ADR-0342 の未決定。
