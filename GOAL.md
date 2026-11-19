# Goal

WASM binding の tests を `wasm/tests.rs` へ切り出す（R6 R2-crates#4 の第一段）。

## Scope

### 背景

`core/src/wasm.rs` は 912 行（本編 533 / tests 379）で、
tests 14 件は `#[cfg(all(test, target_arch = "wasm32"))]` の下にある。
ADR-0255 の EPUB tests 分割と同じ手順で、
公開 API・可視性・test 本文を変えずにファイルだけ分割する。
本編の 4 モジュール分割（epub / archive / lint / support）は
再エクスポート・native cfg・マクロスコープの調整が必要なため、
本 ADR の未決定として残す。

### 成果物

- `core/src/wasm.rs`: 変更（tests module を #[path] 参照へ置換）
- `core/src/wasm/tests.rs`: 追加（既存 tests 本文を移動）
- `docs/decisions/0257-wasm-test-split.md`: 追加
- `todo.md`: R2-crates#4 を「tests 分割完了、本編分割は未決定」へ更新
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. tests 本文を 1 行も変更せず `core/src/wasm/tests.rs` へ移動する。
2. `#[cfg(all(test, target_arch = "wasm32"))] #[rustfmt::skip] #[path = "wasm/tests.rs"] mod tests;` で参照する。
3. 本編 1–533 は変更しない（native / wasm 双方の公開 API を維持）。
4. wasm test 14 件、core 284 件、E2E 255 を維持する。
5. 全ゲート成功。

## Acceptance criteria

- wasm 14 / core 284 / E2E 255、全ゲート成功。

## Constraints

- 本編の 4 モジュール分割は行わない（ADR に未決定として残す）。
- 公開 API / 可視性 / test 内容 / E2E は変更しない。
- `master` へ push する。force push はしない。

## 完了記録

- wasm.rs を 912→537 行、tests.rs へ 376 行・14 tests をバイト単位で移動。
- ADR-0257: 本編 4 モジュール分割、wasm/mod.rs 化、integration test 化は未決定。
- `./scripts/verify.sh`: 全ゲート成功、E2E 255 passed / 0 failed。
