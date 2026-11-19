# Goal

実 Tauri WebView 検証の環境ブロッカーを根拠付きで記録する（R6 A05 実装前提 6）。

## Scope

### 背景

ADR-0272 で desktop 設定保存 UI を接続し、Tauri mock E2E 256/0 を確認した。
しかし実 WebView での往路 / 復路 / 表示 / a11y 検証は未実施だった。
確認の結果、この環境には `cargo-tauri` がインストールされておらず、
`cargo tauri dev` / `build` を実行できない（DISPLAY=:1 / xdotool はある）。

### 成果物

- `docs/decisions/0273-real-webview-verification-deferred.md`: 追加
- `docs/decisions/0272-settings-save-ui.md`: 変更（未決定の具体化）
- `todo.md`: A05 を「mock 完了・実 WebView は環境ブロッカー」へ更新
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. `cargo tauri` 未導入を Runtime Proof として記録する。
2. CDP / Tauri mock は実 WebView の代替ではないことを明記する。
3. Tauri CLI 導入後の検証手順（起動、設定画面、保存往復、a11y）を残す。
4. E2E 256 件・check 名・コードは変更しない。
5. markdownlint 0 issues、verify.sh（256）成功。

## Acceptance criteria

- 文書・索引が整合し、全ゲート成功。

## Constraints

- Tauri CLI のインストール / 外部ダウンロードは行わない。
- 実 WebView を pass と記録しない。
- `master` へ push する。force push はしない。
