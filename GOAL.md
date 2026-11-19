# Goal

リリース運用の導入・更新・ロールバックと既知問題を開示し、署名秘密鍵の誤コミットを防ぐ（R6 R3-release#4,#5）。

## Scope

### 背景

ADR-0222 は updater 未配線を設計受容とし、公開鍵・署名 secrets・
Release workflow・ロールバック手順を未決定として残していた。
外部サービスや成果物公開を行わずに、利用者向けの現状開示と
署名 material の誤コミット防止を先に整備する。

### 成果物

- `.gitignore`: 変更（署名秘密鍵・release secrets の除外パターン）
- `docs/runbook-release.md`: 追加（導入・更新・ロールバック・既知問題）
- `docs/decisions/0254-release-operations-disclosure.md`: 追加
- `todo.md`: R3-release#4,#5 を「運用文書完了、公開作業は未決定」へ更新
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. 現状を「公開 installer / updater / 署名なし」と正直に記載する。
2. 検証済み Web 配布物の導入・更新・ロールバック手順を
   `docs/runbook-verified-build.md` と整合させる。
3. 既知問題（updater 未配線、browser AI 無効、設定読み取り専用、
   実機 5 ブラウザ未検証、CI billing）を根拠 ADR 付きで開示する。
4. `.gitignore` に minisign 秘密鍵・署名 material のパターンを追加する。
5. markdownlint 0 issues、verify.sh（E2E 255）成功。

## Acceptance criteria

- 文書・索引・gitignore が整合し、全ゲート成功。

## Constraints

- Release workflow / CSP / tauri.conf / 公開鍵 / 秘密鍵は作成・変更しない。
- 外部サービス・billing・成果物公開は行わない。
- `master` へ push する。force push はしない。

## 完了記録

- ADR-0254 / runbook-release.md 追加、R4 監査の指摘（公開 endpoint 文言、旧実測例、根拠の重み付け、verify 証跡）を反映。
- `.gitignore` に署名秘密鍵・署名 material の除外パターンを追加（公開鍵は追跡対象）。
- `./scripts/verify.sh`: 全ゲート成功、E2E 255 passed / 0 failed。
- markdownlint: 272 files / 0 issues。
