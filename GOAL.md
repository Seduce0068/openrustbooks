# Goal

CI の gitleaks ゲートが後続 11 ゲートをブロックする問題を修正する（第2cycle R6 / R4-OP01）。

## Scope

### 背景

`.github/workflows/ci.yml` の extra_gates は gitleaks 未導入時に `exit 1` し、
後続の rumdl / dependency / version / e2e-count / csp / plugin / updater /
capability / wasm×2 / wasm-test が到達不能。CI は常時 red のまま回帰を検出できない。

### 成果物

- `.github/workflows/ci.yml`: 変更
- `docs/runbook-verified-build.md`: 変更
- `docs/decisions/0314-gitleaks-gate-order.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. gitleaks 検査を extra_gates の末尾（最終ゲート）へ移し、未導入でも先の静的ゲートが実行されるようにする。
2. 共通 workflow 自体は変更しない。
3. シークレットスキャンは fail-closed を維持する（未導入 / 検出時に最終的に CI は失敗）。
4. billing 解消後チェックリストを更新する。
5. verify.sh 全ゲート成功、E2E 256 件・WDIO 維持。

## Constraints

- 外部サービス / 課金 / 公開操作は行わない。
- `master` へ push する。force push はしない。

## 完了記録

- gitleaks を extra_gates 末尾へ移動し、13 ゲートを先に実行。
- 未導入 / 検出時は最後で fail-closed、共通 workflow は未変更。
- runbook の billing チェックリストを更新。
- verify.sh 全ゲート成功（E2E 256、WDIO 1）。
- CI 側 gitleaks 恒久導入 / CI-H01 は ADR-0314 の未決定。
