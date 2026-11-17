# Goal

`/` キーで章フィルター入力にフォーカスできるようにする。

## Scope

### 背景

章フィルター（ADR-0078）を追加したが、マウスで入力欄をクリックする必要がある。
キーボード操作を補完するため、`/` でフィルターへフォーカスする。

### 成果物

- `web/src/lib.rs`: 変更（キーハンドラ、ヘルプの記載）
- `tools/verify-web-reader.py`: 変更（E2E 検証を追加）
- `docs/decisions/0079-filter-focus-shortcut.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. ショートカット:
   - `/` キーで `#reader-chapter-filter` にフォーカスする。
   - 入力欄にフォーカスがあるときは既存の規則によりキーを奪わない。
   - ヘルプダイアログに `/` の記載を追加する。
2. 既存の挙動を変えない:
   - 既存の 125 テスト、E2E 210 項目を変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - `/` キーでフィルター入力にフォーカスする。

## Acceptance criteria

- 6 ゲートすべて成功する。
- CI は billing issue で実行不可（ローカル検証で代替）。

## Constraints

- 新規依存を追加しない。
- コードは最小限の変更に留める。
- `master` へ push する。force push はしない。
