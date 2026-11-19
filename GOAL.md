# Goal

最終ドキュメント整合（A05 前提 2 / Chrome / Edge 実機 255 / VC-015）。

## Scope

### 背景

監査で次が判明した:
- `todo.md` の A05 前提 2 が ADR-0278 の静的ゲート完了を反映していない。
- ADR-0274 の表は更新済みだが決定節が旧 classification のまま。
- Chrome / Edge 実機 E2E 255/0 は 256 化前の履歴値で、現行 256 件は mock +1 を含む。
- `docs/e2e-traceability.md` の VC-015 が「5 ブラウザ未検証」のまま。

### 成果物

- `todo.md`: 変更（A05 前提 2、Chrome / Edge 履歴表記）
- `docs/decisions/0274-a05-traceability-reconciliation.md`: 変更（決定節の更新）
- `docs/decisions/0265-cross-browser-e2e.md`: 変更（255 を履歴と明記）
- `docs/e2e-traceability.md`: 変更（VC-015 の一部カバー）
- `GOAL.md` / `docs/log.md`: 更新

### 必須事項

1. capability 前提 2 を「静的ゲート完了、実ビルド / 実 WebView 未検証」に統一する。
2. 255 件は当時の履歴、256 件は current（mock +1）と明示する。
3. VC-015 は Chrome / Edge のみ実機確認、残り 3 ブラウザは環境ブロッカーとする。
4. auto gate が参照する `Checker(expected=256)` / CI-H01 / documentation-operations の current 256 は変更しない。
5. markdownlint 0 issues、verify.sh（256）成功。

## Acceptance criteria

- 文書が実態と整合し、全ゲート成功。

## Constraints

- コード / E2E check 名・件数 / Cargo.* は変更しない。
- 外部サービス / 実 WebView 検証は行わない。
- `master` へ push する。force push はしない。
