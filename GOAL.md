# Goal

core の依存のうち feature 差分が無いものを workspace 参照へ統一する（R6 R2-crates#5 の第一段）。

## Scope

### 背景

workspace に既に定義されている依存のうち、core/Cargo.toml が
直接 version を書いているものが残っていた。
feature を足す依存（chrono / uuid / tokio wasm dev）と
featureless ゲート対象（rumdl）は除外し、
完全一致する依存だけを `{ workspace = true }` へ変更する。

### 成果物

- `core/Cargo.toml`: 変更（完全一致依存を workspace 参照化）
- `docs/decisions/0246-workspace-dependency-unification.md`: 追加
- `todo.md`: R2-crates#5 第一段完了、残りは未決定へ
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. workspace と完全一致する依存を `{ workspace = true }` に変更する。
2. 除外: rumdl（featureless gate）、chrono / uuid / tokio wasm dev（feature 加算）、
   workspace に無い依存（zip / quick-xml / imagesize / url 等）。
3. Cargo.lock に差分が出ないこと。
4. check-rumdl-featureless が 2 行を維持すること。
5. 全ゲート成功（E2E 255 維持）。

## Acceptance criteria

- Cargo.lock 差分なし、check-rumdl-featureless 成功、verify.sh 成功。

## Constraints

- feature の加算・削除は行わない。
- 共通ワークフローは変更しない。
- `master` へ push する。force push はしない。
