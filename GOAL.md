# Goal

E2E `wait_until()` の JS 評価例外を握り潰さず fail-closed にする（R6 TD-M02）。

## Scope

### 背景

`wait_until()` は `RuntimeError` のみ再送出し、その他の例外を `pass` で無視する。
JS 式の typo や定数未定義もタイムアウトまで放置され、後続 check が別経路で通れば
E2E 全体が緑になり、検出力が静かに低下する。

### 成果物

- `tools/verify-web-reader.py`: 変更（wait_until の例外伝播、必要なら呼び出し側）
- `docs/decisions/0183-wait-until-failclosed.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. `cdp.evaluate()` 由来の例外は、種別を問わず再送出する（ExecutionContext 破壊など既知の一時例外は既存の retry ラッパー側で扱う）。
2. 既存の E2E 249 項目が壊れないこと。
3. 必要なら一時スクリプトで不正式が即例外になることを確認。
4. 期待総数は 249 のまま。

## Acceptance criteria

- E2E 249 項目、全ゲート成功。

## Constraints

- 新規依存を追加しない。
- 既存 check 名・件数は変更しない。
- `master` へ push する。force push はしない。
