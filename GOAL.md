# Goal

Tauri 設定保存 UI の段階設計を確定する（R6 A05 残、swarm-coding / design）。

## Scope

### 背景

R1 監査で次が判明した:

- `desktop/tauri.conf.json` に `withGlobalTauri` が無く、
  ブラウザ版 WebView から `__TAURI.core.invoke` を取得できない。
- `ConfigView.revision` は返るが更新コマンドが expected を受け取らず、
  CAS 衝突を UI が判別できない（`Result<T, String>` も同じ）。
- 保存 UI を接続するには patch DTO + `get_config` /
  `update_config_patch` の最小 IPC、CSP / capability、
  a11y（label / aria-invalid / aria-describedby）、E2E 成功経路が必要。
- AI / Lint / format の 4 操作は ADR-0232 の初版外で、
  今回の対象に含めない。

### 成果物

- `docs/decisions/0253-tauri-settings-save-design.md`: 追加
- `todo.md`: A05 は優先度「高」のまま「段階設計決定・実装は後続」へ更新
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. browser 単体の読み取り専用 / fail-closed を維持し、desktop のみ保存 UI を接続する方針を記録する。
2. 最小 IPC を `get_config` + `update_config_patch` に限定し、
   fs / ai / window / updater を設定画面から呼ばない旨を明記する。
3. 実装前提（withGlobalTauri、CSP、capability、expected revision、
   構造化エラー、a11y、E2E 成功経路）を ordered list で残す。
4. ADR-0128 / 0193 の desktop 側のみ supersede し、
   browser 側の読み取り専用は維持する。
5. markdownlint 0 issues、verify.sh（E2E 255）成功。

## Acceptance criteria

- 設計 ADR と索引が整合し、全ゲート成功。

## Constraints

- コード・CSP・capability・E2E は変更しない（本次は設計のみ）。
- 外部サービス・署名鍵・秘密情報は扱わない。
- `master` へ push する。force push はしない。

## 完了記録

- ADR-0253 作成、R1 監査（IPC / Web）、R4 設計レビューを実施。
- 採用指摘: capability は独自コマンド非対象の注記、verify.sh 証跡を ADR へ追記。
- `./scripts/verify.sh`: 全ゲート成功、E2E 255 passed / 0 failed。
- markdownlint: 269 files / 0 issues。
