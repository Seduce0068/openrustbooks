# Goal

解決済みの未決定項目が残っている ADR を更新し、ドキュメントの整合性を保つ。

## Scope

### 背景

ADR-0070 〜 0073 の「未決定」には、その後の ADR で既に解消された項目が
残っている。
- ADR-0070: 夏時間の偏移は Date API が自動判定するため追加処理不要（決定済み）
- ADR-0071: JPEG 品質の最適値は ADR-0072 で 0.85 に決定、ADR-0073 で設定化
- ADR-0072: 品質の設定化は ADR-0073 で実装
- ADR-0073: 定量比較は ADR-0074 で調査済み

### 成果物

- `docs/decisions/0070-export-filename-timezone.md`: 変更
- `docs/decisions/0071-downscale-mime-preservation.md`: 変更
- `docs/decisions/0072-downscale-jpeg-quality.md`: 変更
- `docs/decisions/0073-jpeg-quality-setting.md`: 変更
- `GOAL.md`: 本契約

### 必須事項

1. 各 ADR の「未決定」から解消済み項目を「解決済み」セクションへ移す。
   参照先 ADR を明記する。
2. コードは変更しない。既存の 125 テスト、E2E 206 項目を変えない。

## Acceptance criteria

- markdownlint が通る。
- 6 ゲートすべて成功する。

## Constraints

- コードを変更しない。
- `master` へ push する。force push はしない。
