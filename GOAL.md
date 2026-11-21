# Goal

E2E 件数ゲートを Checker(expected=N) の単一情報源へ切り替える（R6 レビュー / R4-CI01）。

## Scope

### 背景

check-e2e-count-consistency.sh は todo.md の未着手 CI-H01 行を要求するため、
CI-H01 を完了セクションへ移すだけで CI が壊れる。
期待件数の出典は tools/verify-web-reader.py の Checker(expected=N) で十分。

### 成果物

- `scripts/check-e2e-count-consistency.sh`: 変更
- `docs/documentation-operations.md`: 必要なら記述更新
- `docs/decisions/0291-e2e-count-single-source.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. todo.md の CI-H01 行有無・内容は検 gate の成功条件にしない。
2. Checker(expected=N) と checker.check/equal 実数の一致は維持する。
3. documentation-operations.md の current 値一致は維持してもよいが、drift 時はどちらが正か明示する。
4. 負向 test: CI-H01 行を削除しても rc=0、Checker 改変は rc!=0。
5. verify.sh 全ゲート成功、E2E 256 件。

## Constraints

- check 名・件数・tools/verify-web-reader.py は変更しない。
- 共通 workflow は変更しない。
- `master` へ push する。force push はしない。
