# Goal

依存更新の自動検出を Dependabot で有効化する（R6 TD-M14）。

## Scope

### 背景

`.github/dependabot.yml` が無く、Cargo 依存と GitHub Actions の更新を手動検出していた。
公式ドキュメントでは `version: 2`、`updates[].package-ecosystem`、
`directory`、`schedule.interval` が必須で、Cargo と GitHub Actions が
サポート対象と明記されている（Dependabot options reference）。

### 成果物

- `.github/dependabot.yml`: 追加（cargo / github-actions、週次、patch/minor グループ）
- `docs/decisions/0215-dependabot.md`: 追加
- `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新
- `todo.md`: TD-M14 を完了へ移動
- `GOAL.md`: 本契約

### 必須事項

1. cargo をルート `/` で週次チェックする。
2. github-actions をルート `/` で週次チェックする。
3. バージョン更新は patch / minor のみを自動 PR 対象とし、
   major は手動判断に残す（セキュリティ更新は Dependabot 既定で継続）。
4. 設定ファイルを YAML として構文検証する。
5. markdownlint 0 issues、既存ゲート（E2E 253）を壊さない。

## Acceptance criteria

- YAML 構文 OK、markdownlint 0 issues、verify.sh 成功。

## Constraints

- 新規依存を追加しない（GitHub 標準機能のみ）。
- 共通ワークフロー `rust-gates.yml@v1` は変更しない。
- 秘密情報・トークンは書かない。
- `master` へ push する。force push はしない。
