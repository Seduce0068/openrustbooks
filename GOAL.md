# Goal

Web 版に Content-Security-Policy を追加し、外部リソースの読み込み先を制限する（todo.md B01 / E03 の一部）。

## Scope

### 背景

`web/index.html` に CSP が無く、inline `<style>` と Google Fonts の外部
スタイルシートが無制限に読み込まれる。CSP を追加して許可先を明示する。

### 成果物

- `web/index.html`: 変更（`<meta http-equiv="Content-Security-Policy">` 追加）
- `tools/verify-web-reader.py`: 変更（CSP 存在の E2E 検証を追加）
- `docs/decisions/0091-content-security-policy.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. 次を許可する CSP を追加する。
   - `default-src 'self'`
   - `script-src 'self' 'wasm-unsafe-eval'`（Wasm のJITに必要）
   - `style-src 'self' 'unsafe-inline' https://fonts.googleapis.com`
   - `font-src 'self' https://fonts.gstatic.com`
   - `img-src 'self' blob: data:`
   - `connect-src 'self'`
2. 現在の E2E 220 項目が通ること。
3. サニタイズ済み本文の相対画像（Blob URL）が表示できること。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- 新規依存を追加しない。
- E2E 環境で外部フォントが読めなくてもテストが壊れないこと。
- `master` へ push する。force push はしない。
