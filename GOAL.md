# Goal

runbook-release / README 細部を現行実装へ追従させる（R6 レビュー / R1-FG01 + R5-DC01）。

## Scope

### 成果物

- `docs/runbook-release.md`: 変更
- `README.md`: 変更
- `docs/decisions/0310-release-doc-current-state.md`: 追加
- `todo.md` / 索引 / log: 更新

### 必須事項

1. updater を「初期化のみ」ではなく「未実装・固定表示・check 禁止ゲート」へ更新する。
2. README の build.rs 配置表記を現行 tree と整合させる。
3. `--locked` 未決定表記を現行ゲート（trunk / Cargo / WDIO / audit freshness）へ更新する。
4. README ADR 要約の誤配置（旧 ADR の要約が欠落 / 重複している行）を修正する。
5. ゲート対象（dependency-hygiene / audit freshness / build-desktop 等）を追記する。
6. verify.sh 全ゲート成功、E2E 256 件・WDIO 維持。

## Constraints

- 公開鍵 / endpoint / release workflow は追加しない。
- 外部サービス / 課金 / 破壊的操作は行わない。
- `master` へ push する。force push はしない。
