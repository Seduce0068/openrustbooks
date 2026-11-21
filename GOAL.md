# Goal

README / 索引 / runbook の現行状態を実装へ追従させる（R6 レビュー / R5-DC01）。

## Scope

### 成果物

- `README.md`: 実 WebView 検証済み、EPUB/ZIP 実装済み、runbook-webdriver 導線
- `docs/index.md`: runbook-webdriver 導線
- `docs/runbook-verified-build.md`: E2E current 256
- `docs/decisions/0265-cross-browser-e2e.md`: current 255 の履歴表記（必要なら）
- `docs/decisions/0273-real-webview-verification-deferred.md`: CLI 導入済み／検証済みの追記
- `todo.md` / `docs/log.md`: 完了移動・追記

### 必須事項

1. README の「実 WebView は未検証」を ADR-0282 / 0283 の完了へ更新する。
2. README の「ドキュメント閲覧 (設計段階): EPUB, ZIP...」を実装済みと未実装を分離する。
3. runbook-verified-build の current E2E を 255 → 256 にする。
4. docs/index / README に runbook-webdriver への導線を追加する。
5. markdownlint 0 issues、verify.sh 全ゲート成功。

## Constraints

- コード / E2E check 名・件数 / Cargo.* は変更しない。
- 外部サービス / 課金 / 公開設定は扱わない。
- `master` へ push する。force push はしない。
