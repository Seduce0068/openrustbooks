# Goal

表示設定を既定値に戻す「Reset settings」ボタンを追加する。

## Scope

### 背景

文字サイズ・行間・ダーク・OS 追従・スワイプ閾値・画像縮小など表示設定が
増えたが、これらを一括で既定に戻す手段が無い。個別に戻すのは手間で、
設定が分かりにくくなったときの復旧手段として必要性が高い。

### 成果物

- `web/src/lib.rs`: 変更（リセット処理とボタン）
- `tools/verify-web-reader.py`: 変更（E2E 検証を追加）
- `docs/decisions/0068-reset-settings.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. リセット処理:
   - ツールバーに `<button id="reader-reset-settings">` を置く。
   - クリックで全表示設定を既定値に戻す:
     fontScale=1, lineHeight=1, dark=false, followSystem=false,
     swipeThreshold=50, verticalScrollThreshold=30, downscaleImages=false。
   - localStorage の設定キーも既定値で上書きする。
   - 状態メッセージに `Display settings reset.` を表示する。
2. 既存の挙動を変えない:
   - 読書位置・履歴・エクスポート・インポートは触らない。
   - 既存の 125 テスト、E2E 193 項目を変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - 設定を変更した後にリセットボタンを押すと、全設定が既定値になる。

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
