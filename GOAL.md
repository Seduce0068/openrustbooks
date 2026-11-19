# Goal

ADR の `generated.at` 意味論を明文化し、日付逆転の誤解を解消する（R6 R5-adr 全体評価）。

## Scope

### 背景

ADR-0001 / 0002 の `generated.at` は 2026-11-17、
ADR-0003〜0005 は 2026-10-16〜17 で見かけ上逆転している。
しかし git 作成順は 0001 → 0002 → 0003 → 0004 → 0005 で連番と一致し、
0001 / 0002 は 2026-11-17 の frontmatter 正規化時に
`generated` が付与されたことが判明した。
よって値の訂正は不要で、規約の注記が必要。

### 成果物

- `docs/documentation-operations.md`: 変更（`generated.at` は生成／正規化時点で作成時刻ではない旨）
- `docs/decisions/0241-generated-timestamp-semantics.md`: 追加
- `todo.md`: R5-adr 全体評価を完了へ移動
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. `generated.at` は生成／正規化時点の記録で、ADR 作成時刻や決定順ではないと明記する。
2. 連番は決定の依存順、作成時刻は git 履歴を参照する旨を記載する。
3. ADR-0001 / 0002 の値は変更しない（正規化時の事実を保持）。
4. markdownlint 0 issues、verify.sh（E2E 254）成功。

## Acceptance criteria

- 規約・ADR・索引が整合し、全ゲート成功。

## Constraints

- 既存 240 件の ADR メタデータは変更しない。
- `created` キーの導入や一括移行は未決定として残す。
- `master` へ push する。force push はしない。
