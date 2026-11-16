# Goal

タッチ操作の縦スクロール判定閾値を設定から変更できるようにする。

## Scope

### 背景

ADR-0056 の未決定「縦スクロール判定閾値（30px）の設定化」。
横スワイプ閾値は設定化済みだが、縦移動をスクロールとみなす 30px は
ハードコードのままで、端末や利用者に合わせられない。

### 成果物

- `web/src/lib.rs`: 変更（`verticalScrollThreshold` の追加と使用）
- `tools/verify-web-reader.py`: 変更（E2E 検証を追加）
- `docs/decisions/0061-vertical-scroll-threshold.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. 設定:
   - `ReaderSettings` に `verticalScrollThreshold` を追加する。
     既定値は 30、範囲は 10〜80px とし、範囲外は既定へ戻す。
   - 既存の設定 JSON にキーが無い場合は既定値 30 として読む（後方互換）。
   - 設定の保存・エクスポート・インポートに含まれる。
2. 動作:
   - 縦スクロール判定に設定値を使う（横スワイプ閾値は維持）。
3. UI:
   - ツールバーに数値入力（`<input id="reader-vertical-threshold">`）を置き、
     変更すると保存・反映される。
4. 既存の挙動を変えない:
   - 既存の 124 テスト、E2E 176 項目を変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - 縦スクロール閾値の入力が存在し、変更すると設定に保存される。
  - 範囲外はクランプされる。

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
