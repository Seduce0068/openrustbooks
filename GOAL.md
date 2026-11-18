# Goal

バックアップのインポートに `window.confirm()` の確認ダイアログを追加する（ADR-0172）。

## Scope

### 背景

ADR-0157 で現在の設定／履歴は保護されたが、読書位置は `savedAt` マージで上書きされ得た。
Clear data には確認を追加済み（ADR-0171）だが、import は未確認だった。

### 成果物

- `docs/decisions/0172-import-confirmation.md`: 追加
- `GOAL.md`: 最終契約へ更新

### 決定

- ファイル取得後・読み込み前に confirm を呼び、false なら input をリセットして return。
- 成功時も input をリセットし同一ファイルを再選択可能にする。
- E2E は既存 import 11 箇所を `select_import_file()` ヘルパーに統一し、
  実ダイアログを回避。期待総数 245 → 246。

## Acceptance criteria

- E2E 246 項目、全ゲート成功（検証済み）。
