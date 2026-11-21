# Goal

openrustbooks に WebdriverIO + @wdio/tauri-service を導入し、実 Tauri WebView を自動操作する最小 E2E を追加する（R6 調査の解消策）。

## Scope

### 背景

Wayland/Xwayland の `xdotool` 入力が不安定なため、Tauri 公式が推奨する
WebDriver ベースの自動化を導入する。既存の CDP/mock E2E 256 件は維持し、
WDIO は別ゲート・別名前空間で追加する。

### 成果物

- `package.json` / `package-lock.json`: 追加
- `tsconfig.json`: 追加
- `e2e/wdio.conf.ts` / `e2e/settings.e2e.ts`: 追加
- `desktop/Cargo.toml` / `desktop/src/main.rs` / `Cargo.lock`: 変更（debug 限定 plugin）
- `desktop/capabilities/default.json`: 追加（debug capability）
- `desktop/tauri.conf.json`: `app.security.capabilities` 追加
- `scripts/verify.sh`: 変更（独立ゲートとして WDIO を追加）
- `docs/decisions/0283-webdriverio-adoption.md`: 追加
- `docs/runbook-webdriver.md`: 追加
- `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md` / `todo.md`: 更新

### 必須事項

1. `@wdio/tauri-service` 1.5.1 と WebdriverIO 9.31 系で導入する。
2. Rust plugin は debug ビルド限定で登録し、release には混入しない。
3. 実 WebView で `/settings` を開き、editable と固定 updater 文言を検証する。
4. `browser.tauri.execute` で `get_config` を呼び、revision を取得する。
5. 既存 E2E 256 件・check 名・`check-e2e-count-consistency.sh` は変更しない。
6. `verify.sh` 全ゲート成功、markdownlint 0 issues。

## Acceptance criteria

- `npm run test:wdio` が成功し、実 Tauri WebView の settings を検証できる。
- 既存 256 件と WDIO が共存する。

## Constraints

- 外部 service / 課金 / 公開ビルドは行わない。
- 既存 CDP E2E と mock の 256 件は変更しない。
- `master` へ push する。force push はしない。

## 完了記録

- WebdriverIO 9.31 系と @wdio/tauri-service 1.5.1 を導入。
- Rust plugin は wdio feature + debugビルド時のみ登録し、capabilityも feature 時のみ読み込む。
- 実 WebView の settings editable・updater 固定文言・get_config IPC を WDIO で検証（1 passing）。
- 既存 E2E 256件・check名は不変。verify.sh 全ゲート成功、wdio gate rc=0。
- CI extra_gates への組み込みとバンドル済み debug バイナリでの実行は未決定。
