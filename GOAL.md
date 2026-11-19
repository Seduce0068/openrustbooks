# Goal

Tauri のグローバル IPC 経路を有効化し、desktop の読み取り表示を可能にする（R6 A05 実装前提 1）。

## Scope

### 背景

`web/src/lib.rs` の invoke ラッパーは `window.__TAURI__.core.invoke` を参照する。
Tauri 2.12.5 の `bundle.global.js` は `window.__TAURI__.core.invoke` を公開するが、
既定の `withGlobalTauri: false` では注入されない。
CSP の `script-src` は Trunk inline script のため既に `'unsafe-inline'` を含み、
グローバルスクリプト追加による CSP 緩和は不要。
npm 依存追加は bundler / lock / オフライン再現性の追加が必要で最小ではない。

### 成果物

- `desktop/tauri.conf.json`: 変更（`app.withGlobalTauri: true`）
- `scripts/check-csp-consistency.sh`: 変更（IPC 経路と withGlobalTauri の整合を静的検査）
- `docs/decisions/0270-tauri-global-ipc-path.md`: 追加
- `docs/decisions/0264-tauri-ipc-path-acceptance.md`: 変更（経路決定を記録）
- `docs/decisions/0253-tauri-settings-save-design.md`: 変更（実装前提 1 の解消）
- `todo.md`: A05 を第一段完了・UI 接続は後続へ更新
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. `withGlobalTauri: true` を追加し、CSP 値は変更しない。
2. 静的ゲートで `withGlobalTauri: true` と web の `__TAURI__.core.invoke` 参照を検査する。
3. browser 単体の fail-closed（`__TAURI__` が無ければ `Err("desktop only")`）は維持する。
4. 実 Tauri WebView での往路検証は未決定として残す（ローカルでは CDP 検証不可）。
5. 全ゲート成功（E2E 255 維持）。

## Acceptance criteria

- 静的ゲート成功、全ゲート成功、E2E 255。

## Constraints

- npm 依存 / `__TAURI_INTERNALS__` 直接利用 / E2E check 追加は行わない。
- 外部サービス・実 WebView 起動は行わない。
- `master` へ push する。force push はしない。
