# Goal

mailto: リンクのクリック動作を E2E で検証する。

## Scope

### 背景

ADR-0081 の未決定。外部リンク（http/https）のポップアップ呼び出しは
検証済みだが、同じ分岐に含まれる `mailto:` は未検証。

### 成果物

- `tools/verify-web-reader.py`: 変更（サンプルに mailto 追加、検証を追加）
- `docs/decisions/0082-mailto-link-test.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. E2E 検証:
   - サンプル CH1 に `mailto:test@example.com` リンクを追加する。
   - サニタイズ後も href が残ること。
   - クリックで `window.open` が `_blank` / `noopener,noreferrer` 付きで
     呼ばれること（ADR-0081 のスタブパターン）。
2. 既存の 125 テスト、E2E 216 項目を変えない。

## Acceptance criteria

- 6 ゲートすべて成功する。
- CI は billing issue で実行不可（ローカル検証で代替）。

## Constraints

- プロダクションコードを変更しない。
- `master` へ push する。force push はしない。
