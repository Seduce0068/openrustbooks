# Goal

cargo audit の advisory DB 鮮度を担保する（R6 レビュー / R3-AU01）。

## Scope

### 背景

verify.sh は `cargo audit --no-fetch` を使い、ローカルキャッシュの
advisory DB に依存する。`--stale` を指定しないため DB が古くても
脆弱性を見逃せる。

### 成果物

- `scripts/verify.sh`: 変更（freshness gate）
- `docs/decisions/0294-audit-db-freshness.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. advisory DB の最終更新が一定期間以内か検査する（例: 7 日）。
2. DB が無い / 古い / 検査不能な場合は fail-closed で失敗する。
3. CI では fetch 有効の audit を使うか、update-db 手順を明示する。
4. verify.sh 全ゲート成功、E2E 256 件・WDIO 維持。

## Constraints

- 脆弱性 DB の自動更新を勝手に行わない（更新はオフライン時失敗し得るため）。
- 共通 workflow は変更しない。
- `master` へ push する。force push はしない。

## 完了記録

- advisory DB 鮮度ゲートを追加（既定7日、上限90日、欠落/未来/不正値は fail-closed）。
- 鮮度検査と cargo audit を同一 DB に固定（`--db $AUDIT_DB_PATH`）。
- 負向 test 14 件 PASS、verify.sh 全ゲート成功（E2E 256、WDIO 1）。
