# Goal

CI / WDIO の再現性と安全性を改善する（R6 レビュー / R4-CI01）。

## Scope

### 成果物

- `scripts/ci-wasm-test.sh`: wasm-pack バージョン固定
- `scripts/ci-wdio-test.sh`: `--locked`、安全な cleanup、一意ログ
- `.github/dependabot.yml`: npm エコシステム追加
- `docs/runbook-webdriver.md`: `npm ci` 統一
- `docs/decisions/0290-ci-reproducibility.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. wasm-pack が 0.13.1 以外なら fail-closed で拒否または強制導入する。
2. wdio build は `cargo build --locked` を使い、lock 変更を検知する。
3. dependabot に npm / root を追加し、WDIO ファミリを group 化する。
4. WDIO cleanup は自プロセス / 自 port のみを扱い、一時ログを使う。
5. runbook は `npm ci` を推奨する。
6. verify.sh 全ゲート成功、E2E 256 件・WDIO ゲート維持。

## Constraints

- 共通 workflow 自体は変更しない。
- 外部サービス / 課金 / 公開設定は扱わない。
- `master` へ push する。force push はしない。
