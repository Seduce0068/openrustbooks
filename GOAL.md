# Goal

CLS（Cumulative Layout Shift）を E2E で直接計測し、画像寸法予約の効果をゲートする。

## Scope

### 背景

ADR-0149 で `width` / `height` 属性による寸法予約を実装したが、
「CLS が減った」ことを直接検証するチェックは無い（ADR-0136 / 0149 の未決定）。
E2E は 234 項目（check 217 + equal 17）。

### 成果物

- `tools/verify-web-reader.py`: 変更（PerformanceObserver による CLS 計測とチェック 1 件追加、234 → 235）
- `docs/decisions/0159-cls-e2e-measurement.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `Page.addScriptToEvaluateOnNewDocument` 等で
   `PerformanceObserver('layout-shift')` を登録し、CLS を蓄積する。
2. サンプル EPUB を開き、画像セトリング後に CLS を読み、
   妥当な閾値（例 0.1 以下）で 1 件チェックする。
3. 閾値は実測に基づき、フレークしない値を選ぶ（最低 5 回連続で確認）。
4. 既存 234 項目を壊さない。期待総数を 235 に更新。

## Acceptance criteria

- E2E 235 項目、5 回連続 0 失敗。
- 6 ゲート + audit + wasm-test が成功する。

## Constraints

- 新規依存を追加しない。
- CLS 計測は本番コードへ影響させない（E2E スクリプトのみ）。
- 外部ブラウザ 5 種の実機検証は本次対象外（Chrome 157 headless のみ）。
- `master` へ push する。force push はしない。
