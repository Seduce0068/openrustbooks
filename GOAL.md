# Goal

`docs/investigation/` の導線を一時調査スナップショットとして明示する（R6 ドキュメント運用）。

## Scope

### 背景

監査で次が判明した:

- `docs/investigation/` は 2026-11-17 の別セッションによる一時調査レポート群。
- ADR-0083 は「変更・コミットしない」と明文で決定している。
- 成果は `todo.md` / ADR / `docs/index.md` に吸収済み。
- 一方 `docs/index.md` の導線は「残タスク調査レポート」とだけ記載され、
  未追跡・スナップショットであることが伝わらない。

### 成果物

- `docs/index.md`: 変更（investigation 導線の説明を明示）
- `docs/decisions/0276-investigation-artifacts-status.md`: 追加
- `todo.md`: 完了反映
- `GOAL.md` / `README.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. ADR-0083 の「別セッション成果物・変更/コミットしない」を再確認して記録する。
2. `docs/index.md` の導線に「未追跡・当時スナップショット・現行値ではない」を明記する。
3. `docs/investigation/` 内のファイルは変更・コミットしない。
4. E2E 256 / check 名 / コードは変更しない。
5. markdownlint 0 issues、verify.sh（256）成功。

## Acceptance criteria

- 導線が実態を反映し、全ゲート成功。

## Constraints

- 外部サービス / 実 WebView 検証は行わない。
- `master` へ push する。force push はしない。

## 完了記録

- ADR-0276 で investigation 生ファイルを一時スナップショットと明示、ADR-0083 の未追跡決定を維持。
- docs/index の導線を「未追跡・非現行値・吸収済み」へ修正、相対リンク 14 件すべて解決。
- `./scripts/verify.sh`: 全ゲート成功、E2E 256 passed / 0 failed。
