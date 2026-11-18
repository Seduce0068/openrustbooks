# Goal

エディタ下書きに容量上限を追加する（ADR-0164 の未決定）。

## Scope

### 背景

ADR-0164 で localStorage ドラフト保存を実装したが、
本文の容量上限が無いため、巨大な入力が QuotaExceededError を
起こし得る（ADR-0154 の保存失敗通知は出るが予防策が無い）。

### 成果物

- `web/src/lib.rs`: 変更（保存前のサイズ検証と固定文言のエラー表示）
- `tools/verify-web-reader.py`: 変更（E2E 1 件追加、241 → 242）
- `docs/decisions/0167-draft-size-limit.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. 下書き本文が上限（例 1 MiB = 1,048,576 bytes）を超える場合、
   `set_item` を呼ばずに保存失敗として扱う。
2. 失敗時は ADR-0165 の `report_save_failure("下書き", SaveErrorTarget::Editor)` を再利用し、
   本文・サイズ値を通知へ載せない（固定文言のみ）。
3. status にも固定文言（例「下書きが大きすぎて保存できません」）を表示する。
4. E2E に 1 件追加（上限超の入力で保存されず失敗通知が出る）。
   期待総数 241 → 242。
5. 既存 241 項目と全ゲートを壊さない。

## Acceptance criteria

- E2E 242 項目が成功する。
- 6 ゲート + audit + wasm-test が成功する。

## Constraints

- 既存の下書きキー／復元挙動は維持する。
- 新規依存を追加しない。
- `master` へ push する。force push はしない。
