# Goal

全 ADR のフロントマターと Markdown 形式を検証し、整合性を確認する。

## Scope

### 背景

79 件の ADR が蓄積された。OKF 準拠のフロントマター（type 必須、
推奨: title / description / tags / generated / status）と markdownlint
準拠が維持されているかを一括検証する。

### 成果物

- `docs/decisions/0001-use-system-webview.md`: 変更（generated 付与、必要に応じて）
- `docs/decisions/0002-ai-markdown-pipeline.md`: 変更（同上）
- `GOAL.md`: 本契約

### 必須事項

1. 検証:
   - 全 ADR に `type` と `status` があること。
   - `generated.at` が RFC 3339（UTC オフセット付き）であること。
   - markdownlint が全ファイルで 0 issues であること。
2. 必要に応じて 0001 / 0002 に generated を付与する（日時は既存の
   コミット日時を推測せず、現在日時を `date` で取得して使用）。
3. コードは変更しない。

## Acceptance criteria

- 全 ADR で検証が通る。
- 6 ゲートすべて成功する。

## Constraints

- コードを変更しない。
- 既存の判断内容は変更しない（メタデータのみ）。
- `master` へ push する。force push はしない。
