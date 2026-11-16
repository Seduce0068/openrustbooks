# Goal

タッチ操作のスワイプ閾値を表示設定から変更できるようにする。

## Scope

### 背景

ADR-0025 の未決定「スワイプの閾値を設定で変更できるようにするか」。
現状は章移動の閾値 50px・縦スクロール判定 30px がハードコードされており、
端末や利用者の操作感に合わせられない。

### 成果物

- `web/src/lib.rs`: 変更（`swipeThreshold` の追加と使用）
- `web/index.html`: 変更（設定 UI のスタイル、必要に応じて）
- `tools/verify-web-reader.py`: 変更（E2E 検証を追加）
- `docs/decisions/0056-swipe-threshold-setting.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. 設定:
   - `ReaderSettings` に `swipeThreshold`（`swipeThreshold`）を追加する。
     既定値は 50。範囲は 30〜120px とし、範囲外は既定へ戻す。
   - 既存の設定 JSON にキーが無い場合は既定値 50 として読む（後方互換）。
   - 設定の保存・エクスポート・インポートに含まれる。
2. 動作:
   - 章移動の横スワイプ閾値を設定値を使う。
   - 縦スクロール判定の 30px は現状維持する。
3. UI:
   - ツールバーに閾値入力（`<input id="reader-swipe-threshold">`）を置き、
     変更すると保存・反映される。
4. 既存の挙動を変えない:
   - 既存の 121 テスト、E2E 169 項目を変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - 閾値入力が存在し、変更すると設定に保存される。
- 既存の 121 テストと E2E 169 項目が通る。

## Acceptance criteria

- 6 ゲートすべて成功する。
- CI は billing issue で実行不可（ローカル検証で代替）。

## Constraints

- 新規依存を追加しない。
- `desktop/` を変更しない。
- プロダクションコードで `unwrap()` / `expect()` / `panic!` / `todo!` を
  使わない。`unsafe` を使わない。
- コメントと doc コメントは日本語で簡潔に書く。
- `master` へ push する。force push はしない。
