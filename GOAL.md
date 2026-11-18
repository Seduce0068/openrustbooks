# Goal

読書位置のインポートに確認ダイアログを追加する（ADR-0171 の未決定）。

## Scope

### 背景

ADR-0157 で現在の設定／履歴は保護されたが、
インポートは新しい読書位置を上書きし得る（ADR-0032 の savedAt マージ）。
Clear data には確認を追加済み（ADR-0171）だが、import は未確認。

### 成果物

- `web/src/lib.rs`: 変更（`window.confirm()` による確認）
- `tools/verify-web-reader.py`: 変更（既存 import テストで confirm を true 化、新規 1 件、245 → 246）
- `docs/decisions/0172-import-confirmation.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. ファイル選択後に `window.confirm()` で確認し、false なら読み込み／取り込みを行わない。
2. キャンセル時は status を変更せず、ファイル入力もリセットして再選択可能にする。
3. 既存 import E2E では confirm を JS オーバーライドで true にし、try/finally で復元。
4. 新規 E2E で confirm=false のとき位置キーが変化しないことを検証。
5. 期待総数 245 → 246。

## Acceptance criteria

- E2E 246 項目が成功する。
- 6 ゲート + audit + wasm-test が成功する。

## Constraints

- ADR-0171 の Clear data confirm と同じ方針（CDP ではオーバーライド）。
- 新規依存を追加しない。
- `master` へ push する。force push はしない。
