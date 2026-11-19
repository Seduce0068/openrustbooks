# Goal

A05 の達成 / 未達をドキュメントへ正確に反映する（R6 要件トレーサビリティ整合）。

## Scope

### 背景

監査で次が判明した:
- A05 前提 1 は IPC 経路のみ完了、nonce/hash と meta/config CSP 一本化は未決定。
- 前提 2（capability 最小権限）は方針のみで、完了証跡なし。
- 前提 3 / 5 は完了、前提 4 は実装済みだが実 WebView の a11y 未検証。
- 前提 6 は Tauri CLI 未導入の環境ブロッカー。
- `docs/runbook-release.md` が「desktop は未接続」と古い現状を記載。
- README の主な機能に desktop 設定保存が未反映。
- ADR-0254 に 0272 / 0273 の事後経緯がない。

### 成果物

- `docs/decisions/0274-a05-traceability-reconciliation.md`: 追加
- `docs/runbook-release.md`: 変更（desktop 接続済み・実 WebView 未検証）
- `README.md`: 変更（主な機能に desktop 設定保存を追加）
- `docs/decisions/0253-tauri-settings-save-design.md`: 変更（前提ごとの状態表）
- `docs/decisions/0254-release-operations-disclosure.md`: 変更（0272 / 0273 の事後経緯）
- `todo.md`: A05 の前提別状態を明記
- `GOAL.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. 前提 1〜6 を「完了 / 一部完了 / 未検証 / 環境ブロッカー」に分類して記録する。
2. A05 全体は完了とせず、実 WebView 検証と capability 完了証跡を残す。
3. Tauri mock E2E 256/0 は JS / IPC 契約レベルであり実 WebView の代替ではないと明記する。
4. E2E check 名・件数・コードは変更しない（256 維持）。
5. markdownlint 0 issues、verify.sh（256）成功。

## Acceptance criteria

- 文書・索引が整合し、全ゲート成功。

## Constraints

- コード / E2E / Cargo.* は変更しない。
- 外部サービス / 実 WebView 検証は行わない。
- `master` へ push する。force push はしない。
