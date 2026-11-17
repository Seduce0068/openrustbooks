# Goal

ADR-0081 の未決定を後続 ADR の解決結果へ更新する（todo.md C07）。

## Scope

### 背景

ADR-0081 の「未決定」に `mailto:` リンクのクリック動作の検証が残っているが、
ADR-0082 で実装・検証済み。ドキュメントの整合性を保つ。

### 成果物

- `docs/decisions/0081-external-link-popup-test.md`: 変更
- `GOAL.md`: 本契約

### 必須事項

1. `mailto:` の項目を「未決定」から「解決済み」へ移し、ADR-0082 を参照する。
2. 他の未決定（遅延読み込み、EPUB 内 CSS 等）は維持する。
3. コードは変更しない。

## Acceptance criteria

- markdownlint が通る。
- 6 ゲートが成功する。

## Constraints

- コードを変更しない。
- `master` へ push する。force push はしない。
