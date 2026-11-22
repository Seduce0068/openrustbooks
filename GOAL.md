# Goal

イベント発行失敗と import 混在状態を可視化する（R6 レビュー / R1-ER01）。

## Scope

### 成果物

- `desktop/src/commands/fs.rs`, `desktop/src/commands/document.rs`, `desktop/src/main.rs`: 変更
- `web/src/lib.rs`: 変更
- `docs/decisions/0307-event-emit-visibility.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. `let _ = app.emit(...)` / `window.emit(...)` を警告ログ化し、失敗を黙って捨てない。
2. import の成功 / 失敗件数を固定文言で status へ返す（実パス / 値は出さない）。
3. 目次 / landmark / page list の解析失敗も固定文言で status 化する。
4. E2E 256 件・check 名・必須文言を維持する。
5. verify.sh 全ゲート成功。

## Constraints

- 外部サービス / 秘密情報は扱わない。
- 新たな E2E check は追加しない（256 件維持）。
- `master` へ push する。force push はしない。

## 完了記録

- emit 失敗を警告ログ化、import 失敗件数と nav parse 失敗を status 化。
- E2E 必須の成功文言 / settings / updater / a11y 文言を維持。
- 478 tests、verify.sh 全ゲート成功（E2E 256、WDIO 1）。
- UI status への emit 失敗伝播 / parse 文言粒度は ADR-0307 の未決定。
