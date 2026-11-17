# Goal

REQ-010 の「最大 3 回再生成」を実装と一致する文言に明文化する（ADR-0105 / 0120 の未決定）。

## Scope

### 背景

`docs/requirements.md:132` は「rumdl 検査合格まで最大3回再生成」とする一方、
実装は `for attempt in 0..=max_retries`（既定 3）で
初期 1 回＋リトライ 3 回＝最大 4 回生成する。
ADR-0105 / 0120 で解釈差が未決定として残っている。

### 成果物

- `docs/requirements.md`: 変更（文言を明文化）
- `docs/decisions/0121-req010-retry-wording.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. REQ-010 の品質ゲートを「初期生成 1 回＋リトライ最大 3 回（合計最大 4 回）」と明記する。
2. 上限設定 `max_retries` の既定 3 と範囲 0..=10（ADR-0120）への言及を追加する。
3. 既存の「失敗時は差分提示しユーザー判断へ」は維持する。
4. コードは変更しない。

## Acceptance criteria

- markdownlint が通る。

## Constraints

- コードを変更しない。
- 実装の挙動（最大 4 回生成）は変更しない。
- `master` へ push する。force push はしない。
