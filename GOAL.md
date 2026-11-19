# Goal

meta CSP と desktop config CSP の差分を静的ゲートで検出する（ADR-0130 の未決定を解消）。

## Scope

### 背景

`web/index.html` の meta CSP と `desktop/tauri.conf.json` の config CSP は
意図的に allowlist が異なる（desktop は ipc / OpenAI / Ollama を許可）。
ADR-0130 は desktop 側 CSP を直接検証する静的ゲートを未決定として残していた。

### 成果物

- `scripts/check-csp-consistency.sh`: 追加
- `scripts/verify.sh`: 変更（静的ゲート追加）
- `.github/workflows/ci.yml`: 変更（extra_gates に 1 行）
- `docs/decisions/0268-csp-consistency-gate.md`: 追加
- `docs/decisions/0130-desktop-csp.md`: 変更（未決定の解消を記録）
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. meta CSP / config CSP を解析し、script-src / style-src が一致することを検査する。
2. config CSP に `default-src` / `img-src` / `font-src` / `connect-src` が存在することを検査する。
3. meta CSP に外部オリジン（http/https）が無く、config に IPC / 既知 AI endpoint 以外の外部オリジンが無いことを検査する。
4. 負向 test（不一致 / 欠落 / 未知オリジン / 不正 JSON / ファイル欠落）で失敗する。
5. 全ゲート成功（E2E 255 維持）。

## Acceptance criteria

- 負向 test が失敗し、通常 run で全ゲート成功。

## Constraints

- CSP / HTML / tauri.conf / E2E check 名・件数は変更しない。
- 共通ワークフローは変更しない。
- `master` へ push する。force push はしない。

## 完了記録

- CSP 整合ゲートを追加し script-src / style-src、必須 directive、許可オリジンを fail-closed 検査。
- 監査指摘（未知 directive / 'unsafe-eval' 同時緩和）を反映、負向 test 15/15 PASS。
- ADR-0268 で ADR-0130 の静的ゲート未決定を解消、E2E 255/0 維持。
