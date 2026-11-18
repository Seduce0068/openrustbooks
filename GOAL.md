# Goal

`scripts/verify.sh` の配布 WASM パス検査を fail-closed にする。

## Scope

### 背景

5 観点の残タスク探索で、現在の検査が
`grep -a -q '/home/' "$DIST_DIR"/*_bg.wasm` であり、
glob が非マッチなら `grep` が rc=2 を返すものの `if` 条件で偽として扱われ、
ゲートを通過する fail-open 状態にあることが判明した（discovery-test / deps-001 系）。

### 成果物

- `scripts/verify.sh`: 変更（ファイル存在確認と固定文字列検査）
- `docs/decisions/0151-remaining-task-discovery.md`: 追加（探索結果の集約）
- `docs/decisions/0152-wasm-path-check-failclosed.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `nullglob` または配列代入で `*_bg.wasm` の一致ファイル数を確認し、
   0 件なら明示的に失敗させる。
2. 検査対象に生成 JS グルーコード（`*.js`）も含める。
3. 開発ホスト絶対パスは `$HOME/` を固定文字列で検査し、
   `$HOME` が `/home/` 配下でない環境でも検出できるようにする。
4. 既存の E2E 231 項目と全ゲートを壊さない。
5. `bash -n` / `shellcheck` / `shfmt -d` が通る。
6. 負テスト（glob 非マッチ時に失敗すること）を確認する。

## Acceptance criteria

- 全ゲート成功、E2E 231 項目成功。
- bash -n / shellcheck / shfmt が 0。

## Constraints

- 新規依存を追加しない。
- `todo.md` と `docs/investigation/` は別セッションの成果物のため触らない。
- `master` へ push する。force push はしない。
