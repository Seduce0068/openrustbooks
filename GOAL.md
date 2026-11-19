# Goal

ADR-0232 の監査指摘を修正した（初版スコープ ADR の精度向上）。

## Scope

### 成果物

- `docs/decisions/0232-initial-release-scope.md`: 更新（誤引用・TXT・行番号・Runtime Proof）
- `GOAL.md` / `docs/log.md`: 最終状態へ同期

### 決定

- 行番号参照を実測値へ更新し、誤引用（ADR-0229/0227/0224）を訂正。
- TXT 編集を初版外へ移动し、browser は Markdown のみと明記。
- requirements.md の未決定は継続有効と明記。
- Runtime Proof に lint 条件と verify.sh の結果を記録。

## Acceptance criteria

- markdownlint 0 issues、verify.sh 成功（検証済み）。
