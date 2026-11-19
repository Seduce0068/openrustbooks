# Goal

CI / ローカルゲートの整合を確認し、`trunk build --locked` を追加した（R6 R3-ci#3,#5）。
**完了（2026-11-19）**。

## Scope

### 背景

`wasm_targets` は共通ワークフローで target 導入のみを行い、
実検査は `extra_gates` が担うため二重実行ではない。
CI の `CARGO_PROFILE_*` 環境変数は GitHub runner のディスク対策で、
ローカル verify.sh に全局適用すべきではない。
一方 `scripts/verify.sh` の `trunk build --offline` に `--locked` が無く、
`Cargo.lock` を黙って更新できる fail-open が残っていた。

### 成果物（完了）

- `scripts/verify.sh`: 変更済み（`trunk build --offline --locked --dist ...`、268 行目）
- `docs/decisions/0237-ci-gate-consistency.md`: 追加済み
- `GOAL.md` / `todo.md` / README / 各 index / `docs/log.md`: 完了反映済み

### 必須事項（すべて充足）

1. `trunk build` に `--locked` を追加（`--offline` は維持）。→ 完了
2. `ci.yml` / `ci-wasm-test.sh` は変更しない（二重実行ではないため）。→ 完了
3. ADR に wasm_targets と extra_gates の役割分担、
   ビルドプロファイル差、env をローカルへ適用しない理由を記録。→ 完了
4. markdownlint 0 issues、verify.sh（E2E 254）成功。→ 完了

## Acceptance criteria

- [x] verify.sh が `--locked` 付きで全ゲート成功（rc=0、E2E 254/0）。
- [x] `bash -n scripts/verify.sh` OK。
- [x] `Cargo.lock` 差分なし。
- [x] ADR-0237 を追加し、全索引を 237 件へ更新。
- [x] markdownlint 0 issues。

## Constraints / 未決定

- 共通ワークフロー自体は変更しない（監査の結果、修正不要と判定）。
- 外部サービス・billing の状態は未決定として残す（CI-H01 は低優先度で継続）。
- 本番プロファイル（`--release` vs dev）、env の trunk 行スコープ、
  CI での trunk build / E2E 実行、`--frozen` 統一、billing 解消後の実 CI 検証は
  ADR-0237 の「未決定」へ集約する。
- `master` へ push する。force push はしない（本エージェントは git 操作禁止、親が実施）。
