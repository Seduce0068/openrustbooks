# Goal

検証済み配布物をローカルへ保持し、復旧の糸口を作る（R6 TD-M16 の即時対応）。

## Scope

### 背景

`scripts/verify.sh` は `trunk build --offline` を `mktemp -d` へ出力し、
`trap` で毎回削除するため、検証成功後の配布物が残らない。
`web/dist` は git 管理外で、現行物は開発パス残存の可能性があり
「検証済み」ではない。CI の共通 `rust-gates.yml@v1` は
artifact upload 入力を持たず、billing issue も未解消。

### 成果物

- `scripts/verify.sh`: 変更（`--dist <dir>` オプション、成功時のみコピーと manifest 生成）
- `docs/runbook-verified-build.md`: 追加（再利用／復旧手順）
- `docs/decisions/0219-verified-artifact-retention.md`: 追加
- `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新
- `todo.md`: TD-M16 を完了（即時対応完了・恒久策は未決定）へ移動
- `GOAL.md`: 本契約

### 必須事項

1. `./scripts/verify.sh --dist <dir>` で、全ゲート成功時のみ
   検証済み dist を `<dir>/<short-sha>-<UTC>/` へコピーする。
2. manifest に commit sha、rustc / trunk バージョン、
   dist ファイルの SHA256、E2E 期待／結果、検証日時を記録する。
3. ゲート失敗時はコピーしない。既定の動作（引数なし）は従来どおり一時 dist を削除する。
4. runbook に再取得／SHA 検証／serve／復旧手順を記載する。
5. markdownlint 0 issues、引数なし／`--dist` 両方の verify.sh が成功する。

## Acceptance criteria

- 両 verify.sh が成功し、manifest と成果物が生成される。

## Constraints

- 67MB の wasm を git 追跡しない（`.gitignore` 維持）。
- 新規依存・外部送信・破壊的操作を追加しない。
- 共通ワークフロー `rust-gates.yml@v1` は変更しない。
- `master` へ push する。force push はしない。
