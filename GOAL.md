# Goal

本文内の外部リンクが安全に開かれることを E2E で検証する。

## Scope

### 背景

ADR-0013 で「外部リンクは opener を渡さずに開く」と決定し、
`window.open(..., "_blank", "noopener,noreferrer")` を実装しているが、
E2E で検証していない。サンプルに外部リンクを追加して回帰テストにする。

### 成果物

- `tools/verify-web-reader.py`: 変更（サンプルに外部リンク追加、検証を追加）
- `docs/decisions/0080-external-link-safety-test.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. E2E 検証:
   - サンプル CH1 に `https://example.com` へのリンクを追加する。
   - サニタイズ後も href が残ること。
   - リンクが `rel="noopener noreferrer"` を持つこと（ammonia の link_rel）。
   - javascript: リンクは除去される既存テストを維持する。
2. 既存の 125 テスト、E2E 212 項目を変えない。

## Acceptance criteria

- 6 ゲートすべて成功する。
- CI は billing issue で実行不可（ローカル検証で代替）。

## Constraints

- プロダクションコードを変更しない。
- `master` へ push する。force push はしない。
