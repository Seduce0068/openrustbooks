# Goal

Web 版 EPUB リーダーを Windows のハイコントラスト（Forced Colors Mode）に対応させる。

## Scope

### 背景

ADR-0038 まででライト/ダークと `prefers-color-scheme` 追従に対応したが、
Windows のハイコントラスト（強制カラーモード）ではブラウザが作者指定の色を
上書きし、ボタンや境界の視認性が失われる恐れがある
（ADR-0033 の未決定「`forced-colors` や `prefers-contrast` への対応」）。

### 成果物

- `web/index.html`: 変更（`@media (forced-colors: active)` のシステムカラー指定）
- `tools/verify-web-reader.py`: 変更（強制カラーモードの E2E 検証を追加）
- `docs/decisions/0039-forced-colors.md`: 追加（決定と検証の記録）
- `GOAL.md`: 本契約

### 必須事項

1. Forced Colors Mode:
   - `@media (forced-colors: active)` で、主要な色トークンをシステムカラー
     キーワード（`Canvas` / `CanvasText` / `ButtonText` / `Highlight` /
     `GrayText` 等）に上書きする。
   - ボタン・入力欄の境界は `border` または `outline` でシステムカラーを
     使い、背景色や box-shadow だけに依存しない。
   - `:focus-visible` のアウトラインが強制カラーモードでも消えないようにする。
2. 既存の挙動を変えない:
   - 既存の 119 テスト、E2E 140 項目、読書位置、履歴、表示設定、掃除、
     キーボード操作、タッチ操作、landmarks、page-list、履歴一覧、
     履歴クリア・個別削除、エクスポート・インポート、マージ、
     カラースキーム追従、画像の非同期デコード、スクロール順序テストを
     変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - 強制カラーモードをエミュレーションしてもリーダーが表示され、
    主要な操作（章移動）ができること。
- 既存の 119 テストと E2E 140 項目が通る。

## Acceptance criteria

- 上記の必須事項がコードと E2E で確認できる。
- 6 ゲートすべて成功する。
- CI は billing issue で実行不可（ローカル検証で代替）。

## Constraints

- 新規依存を追加しない。外部送信を追加しない。
- `desktop/` を変更しない。
- コメントと CSS は日本語で簡潔に書く。
- 無関係なリファクタリングをしない。
- `master` へ push する。force push はしない。
