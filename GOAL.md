# Goal

CI ゲートの欠落を解消する（R6 レビュー / R4-CI01）。

## Scope

### 成果物

- `scripts/check-no-external-font-refs.sh`: 追加（verify.sh の inline gate を抽出）
- `scripts/check-distribution-path.sh`: 追加（--dist 絶対パス / repo 外 / manifest 形式を検査）
- `scripts/check-secrets.sh`: 追加（gitleaks を pre-commit 主軸、CI も final gate）
- `.pre-commit-config.yaml`: 追加（gitleaks hook）
- `scripts/verify.sh`:  inline gate を script 呼び出しへ変更、secret gate 追加
- `.github/workflows/ci.yml`: extra_gates に静的 gate を追加
- `docs/runbook-ci.md` または `docs/runbook-verified-build.md`: billing 解消後チェックリスト
- `docs/decisions/0311-ci-gate-coverage.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. no-external-font-refs / 配布パス / シークレットスキャンを CI の extra_gates で実行する。
2. gitleaks は pre-commit を主軸とし、CI でも final gate として実行する。
3. trunk build + E2E / WDIO は重いため billing 解消後チェックリストに明示する（勝手に common workflow を変更しない）。
4. E2E 256 件・check 名・verify.sh 全ゲート成功。

## Constraints

- 共通 workflow（Seduce0068/workflows）自体は変更しない（ローカル ci.yml の extra_gates のみ）。
- 外部サービスへの課金・公開操作は行わない。
- `master` へ push する。force push はしない。

## 完了記録

- no-external-font / 配布パス + manifest / gitleaks を verify.sh と CI へ配線。
- gitleaks は pre-commit 主軸 + CI 最終ゲート（未導入 fail-closed）。
- 共通 workflow は未変更、billing 後チェックリストを runbook に明記。
- R4 指摘すべて反映、verify.sh 全ゲート成功（E2E 256、WDIO 1）。
- 期待値動的取得 / self_test trap / CI gitleaks 導入は ADR-0311 の未決定。
