# Goal

デスクトップ版の CSP を Web 版と整合させ、`'unsafe-eval'` を排除する（ADR-0091 / 0128 / 0129 の未決定）。

## Scope

### 背景

`desktop/tauri.conf.json:29` の CSP は `script-src 'self' 'unsafe-inline' 'unsafe-eval'` で、
Wasm に必要な `'wasm-unsafe-eval'` が無い。一方 `web/index.html:7` には
`'wasm-unsafe-eval'` がある。ADR-0091 で nonce 対応は見送られており、
`'unsafe-inline'` は残る。

### 成果物

- `desktop/tauri.conf.json`: 変更（`'unsafe-eval'` → `'wasm-unsafe-eval'`）
- `docs/decisions/0130-desktop-csp.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `script-src` から `'unsafe-eval'` を削除し、`'wasm-unsafe-eval'` を追加する。
2. `connect-src` 等は変更しない（Tauri IPC と既存送信先を維持）。
3. ローカル検証（trunk build / E2E）が壊れないことを確認する。
4. 実機での Tauri WebView 挙動は未検証であることを ADR に明記する。

## Acceptance criteria

- 6 ゲート + audit + wasm-test + E2E が成功する。

## Constraints

- `withGlobalTauri` は今回追加しない（別タスク）。
- `web/index.html` の CSP は変更しない。
- `master` へ push する。force push はしない。
