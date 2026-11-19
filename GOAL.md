# Goal

ADR-0232 の監査指摘を修正する（初版スコープ ADR の精度向上）。

## Scope

### 背景

監査で ADR-0232 に対し、根拠 file:line の陳腐化、
TXT editor の過大主張、Runtime Proof の記録不足が指摘された。
誤引用（ADR-0229 / 0227 / 0224）は既に反映済みだった。

### 成果物

- `docs/decisions/0232-initial-release-scope.md`: 変更
- `docs/log.md`: 変更（任意）
- `GOAL.md`: 本契約

### 必須事項

1. 行番号参照を節見出しへ変更し、陳腐化を防ぐ。
2. TXT editor を初版外と明記し、browser では Markdown のみと整理する。
3. Runtime Proof に親が実施した verify.sh（E2E 254/0）と markdownlint 0 issues を記録する。
4. requirements.md の未決定が継続有効である旨を維持する。
5. markdownlint 0 issues。

## Acceptance criteria

- markdownlint 0 issues、変更後も verify.sh 成功。

## Constraints

- 範囲そのものは変更しない（記載精度のみ）。
- `master` へ push する。force push はしない。
