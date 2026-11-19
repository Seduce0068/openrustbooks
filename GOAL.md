# Goal

設定画面の browser-only 説明を desktop では非表示にする（R6 A05 UX 整合）。

## Scope

### 背景

ADR-0272 で desktop の設定保存 UI が接続されたが、
`web/src/lib.rs:4358` の「ブラウザ版は表示のみです…」は常時表示のまま。
desktop で編集可能なときに誤解を与えるため、
browser の readonly 契約を維持しつつ desktop では非表示にする。

### 成果物

- `web/src/lib.rs`: 変更（browser-only 通知の条件表示）
- `tools/verify-web-reader.py`: 変更（既存 mock check 内で desktop の非表示を検証、check 数不変）
- `docs/decisions/0275-settings-notice-visibility.md`: 追加
- `todo.md`: 設計補完として完了反映
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. browser（editable=false）では既存の説明文を表示する。
2. desktop（editable=true）では当該説明文を表示しない。
3. 既存 E2E 2 件（fallback / readonly）と Tauri mock check は check 名・件数を維持する。
4. E2E 256 件・check 名・コード出典を変更しない。
5. markdownlint 0 issues、verify.sh（256）成功。

## Acceptance criteria

- browser E2E と desktop mock の両方で正しい表示shield、全ゲート成功。

## Constraints

- 構造化エラー / DTO / IPC / a11y 属性は変更しない。
- 外部サービス / 実 WebView 検証は行わない。
- `master` へ push する。force push はしない。

## 完了記録

- browser-only 設定説明を editable=false 時のみ表示する <Show> 条件表示へ変更。
- Tauri mock check に desktop での非存在を追加（check 名・件数 256 不変）。
- ADR-0275 で ADR-0193 / 0253 の常時説明契約を browser 限定へ更新。
- /tmp 満杯による E2E 失敗は古い target キャッシュ削除後に再検証し 256/0 を確認。
