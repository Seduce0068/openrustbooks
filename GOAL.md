# Goal

updater 静的ゲートの `.check()` 検出漏れを修正する（R6 R1 で発見した fail-open の解消）。

## Scope

### 背景

R1 監査で、2 つの静的ゲートが `UpdaterExt::check` と
`.check_update(` などしか検出せず、Tauri v2 の実際の呼び出しである
`updater.check().await` と `app.updater()?.check().await` を検出しないことが分かった。
このままだと、更新チェックを配線してもゲートが成功する。

### 成果物

- `scripts/check-updater-honesty.sh`: 変更（`.check(` 検出を追加）
- `scripts/check-capability-scope.sh`: 変更（`.check(` 検出を追加）
- `docs/decisions/0279-updater-check-pattern.md`: 追加
- `todo.md`: 完了反映
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. 両スクリプトで `.check(` を検出する。
2. `.check_updates(` などの正当なフィールド名と誤爆しない。
3. 既存の負向 test を維持し、`.check()` の負向 test を追加する。
4. E2E 256 件・check 名を変更しない。
5. markdownlint 0 issues、verify.sh（256）成功。

## Acceptance criteria

- `.check()` を入れた一時コピーが rc=1 になり、現行ツリーが rc=0 になる。
- 全ゲート成功。

## Constraints

- desktop / web / Cargo.* / E2E スクリプトは変更しない。
- 公開鍵 / endpoint / 署名 secrets は導入しない。
- `master` へ push する。force push はしない。

## 完了記録

- 両スクリプトに `.check(` と capability-scope に `.check_updates(` を追加。
- 一時コピーによる負向 test 3 ケースすべて rc=1、現行ツリー rc=0。
- ADR-0279 を作成し、0277 / 0278 の検出対象を更新。
- verify.sh 全ゲート成功、E2E 256 passed / 0 failed。
