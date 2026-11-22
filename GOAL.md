# Goal

依存ゲートを強化する（R6 レビュー / R2-DP01）。

## Scope

### 背景

`check-rumdl-featureless.sh` は Cargo.toml の featureless 宣言のみを検査し、
Cargo.lock に残る serde_yaml / tikv-jemalloc-sys を検出しない。
cargo audit は unmaintained 警告を許容しており、
既知例外の根拠・期限管理が無い。

### 成果物

- `scripts/check-dependency-hygiene.sh`: 追加
- `scripts/check-rumdl-featureless.sh`: 必要なら変更
- `scripts/verify.sh`: ゲート配線
- `docs/decisions/0309-dependency-gate-hardening.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. `cargo tree -i <crate>` で serde_yaml / tikv-jemalloc-sys の必須依存経路を検出する。
2. 既知例外は ID / 理由 / 期限を管理し、期限切れは fail-closed にする。
3. 未使用依存の lock 残存を警告ではなく失敗にする（例外は明示）。
4. verify.sh にゲートを追加し、全ゲート成功・E2E 256 件を維持。

## Constraints

- 依存そのものの更新 / 削除は行わない（既知 baseline の扱いのみ）。
- 外部サービス / 課金 / 公開設定は扱わない。
- `master` へ push する。force push はしない。

## 完了記録

- dependency-hygiene ゲートを追加し、cargo tree 実測 + 期限付き例外を管理。
- verify.sh と CI extra_gates の両方へ配線。
- 負向 test 5 ケースすべて rc=1、全ゲート成功（E2E 256、WDIO 1）。
- unmaintained 警告の個別管理は ADR-0309 の未決定。
