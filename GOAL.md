# Goal

Tauri IPC 経路の現状を設計受容として ADR に記録する（R6 A05 第一段の設計補強）。

## Scope

### 背景

R1 監査で、現状の `web/src/lib.rs` が `window.__TAURI.core.invoke` を参照する一方、
`desktop/tauri.conf.json` に `withGlobalTauri: true` が無いため、
desktop でも `fetch_config()` が `Err("desktop only")` になる可能性が判明した。
代替経路（npm / `__TAURI_INTERNALS__` / withGlobalTauri）は
それぞれ公式契約・最小差分・CSP 設計のトレードオフがあり、
実装前に設計判断を記録する。

### 成果物

- `docs/decisions/0264-tauri-ipc-path-acceptance.md`: 追加
- `docs/decisions/0253-tauri-settings-save-design.md`: 変更（現状と未決定を補強）
- `todo.md`: A05 は設計補強完了・実装は後続へ更新
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. desktop も IPC が機能していない可能性を正直に記録する。
2. 3 経路（npm / internals / withGlobalTauri）の比較と未決定を残す。
3. CSP nonce/hash、meta/config CSP 一本化、静的ゲート、実 WebView 検証を未決定に含める。
4. コード・CSP・capability・E2E は変更しない。
5. markdownlint 0 issues、verify.sh（E2E 255）成功。

## Acceptance criteria

- 文書・索引が整合し、全ゲート成功。

## Constraints

- 外部サービス・実 Tauri WebView 検証は行わない。
- `master` へ push する。force push はしない。

## 完了記録

- ADR-0264 で desktop IPC 未接続の可能性と 3 経路比較を記録（実機未検証）。
- ADR-0253 / todo に追跡先を追記、ark 監査の軽微指摘（CSP 使い分け、リンク、重複行）を修正。
- `./scripts/verify.sh`: 全ゲート成功、E2E 255 passed / 0 failed。
