# Goal

古い TODO を現 HEAD の実態へ合わせて整理する（stale todo cleanup）。

## Scope

### 背景

ADR が 223 件まで進んだ一方、todo.md の未着手には
解消済みの高／中優先度項目が残っている。
監査で B03 / E01 / D06 / R4-e2e#1 / A09 / Tauri パス検証などが
ADR とコードにより解消済みと判定された。

### 成果物

- `todo.md`: 変更（解消済み項目を完了へ移動／削除、CI-H01 を外部要因依存へ）
- `docs/decisions/0224-todo-reconciliation.md`: 追加
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. 解消済みと判定された項目を未着手から削除し、完了へ移動する。
2. CI-H01 は「billing 解消後の実 CI 検証」として外部要因／低優先度へ移動し、E2E 件数を 246 → 253 に修正する。
3. 重複する残リスクは ADR の「未決定」を参照先として一本化する。
4. 既存の履歴 ADR / 完了エントリは保持する。
5. markdownlint 0 issues、verify.sh（E2E 253/0）成功。

## Acceptance criteria

- markdownlint 0 issues、verify.sh 成功。

## Constraints

- 新規 ADR 以外の設計判断は行わない（監査結果の反映のみ）。
- コード・E2E は変更しない。
- `master` へ push する。force push はしない。
