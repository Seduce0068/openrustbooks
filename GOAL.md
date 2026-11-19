# Goal

E2E トレーサビリティ文書を現在の 253 件へ更新する（ADR-0206 の追従漏れ）。

## Scope

### 背景

ADR-0206 / `docs/e2e-traceability.md` は 251 件の対応表を作成したが、
その後に 2 件追加された（ADR-0212: エラー redaction、
ADR-0213: prefers-reduced-motion）。
現在の `Checker(expected=253)` と対応表が不一致で、
要件から E2E への追跡が途中までしかできない。

### 成果物

- `docs/e2e-traceability.md`: 変更（253 件、2 件の新規 check を対応表へ）
- `docs/index.md`: 変更（説明の件数 251 → 253）
- `docs/decisions/0220-traceability-refresh.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映
- `README.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. AST で `tools/verify-web-reader.py` の check 名 253 件を抽出し、
   対応表の check 名と完全一致（missing / duplicate / extra 0）にする。
2. 新增 2 件（不正 EPUB の固定文言、reduce の即時移動）を適切な VC へ割り当てる。
3. `Checker(expected=253)` をコード出典として明記する（ADR-0090 を維持）。
4. 履歴 ADR / 完了 todo の 251 件表記は作成時点の実績として保持する。
5. markdownlint 0 issues、verify.sh（E2E 253/0）成功。

## Acceptance criteria

- AST 一致 253/253、markdownlint 0 issues、verify.sh 成功。

## Constraints

- E2E コード・check 名・期待総数は変更しない。
- 新規依存を追加しない。
- `master` へ push する。force push はしない。
