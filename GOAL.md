# Goal

Google Fonts の外部送信を削除する（R6 PR-M07、ADR-0191）。

## Scope

### 成果物

- `web/index.html`: preconnect / stylesheet 削除、CSP と font stack 変更
- `scripts/verify.sh`: no-external-font-refs ゲート追加
- `docs/decisions/0191-external-font-removal.md`: 追加
- `todo.md`: PR-M07 を完了へ移動

### 決定

- 第三者フォントを完全削除し、システムフォントスタックへ移行。
- CSP から external font host を削除。
- verify.sh に静的ゲートを追加（一致で fail-closed）。
- web/dist は再ビルド済みだが git 追跡対象外。

## Acceptance criteria

- E2E 249 項目、全ゲート成功（検証済み）。
