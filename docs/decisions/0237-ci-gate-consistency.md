---
type: Decision
title: "ADR-0237: CI / ローカルゲートの整合と trunk build への --locked 付与"
description: "wasm_targets と extra_gates の役割分担を確認し、verify.sh の trunk build に --locked を付けて Cargo.lock の不変を保証する決定。"
tags:
  - ADR
  - ci
  - gates
  - trunk
  - lockfile
generated:
  by: dsh
  at: "2026-11-19T19:13:18+09:00"
status: accepted
---

## 現状

- `.github/workflows/ci.yml:38` の `wasm_targets: wasm32-unknown-unknown` は、共通ワークフロー
  `rust-gates.yml@v1` の「追加ターゲット（wasm32 等）」ステップで
  `rustup target add "$target"` を実行するだけである。実検査は行わない。
- 同じく `ci.yml:46-50` の `extra_gates` が `check-rumdl-featureless.sh`、
  `cargo check -p openrustbooks-core --target wasm32-unknown-unknown --locked`、
  `cargo check -p openrustbooks-web --target wasm32-unknown-unknown --locked`、
  `ci-wasm-test.sh` を「追加ゲート」ステップで実行する。wasm 対象の導入と検査は
  責務が分かれており、二重実行ではない。
- `ci.yml:42-45` の `CARGO_PROFILE_DEV_DEBUG=0` / `CARGO_PROFILE_TEST_DEBUG=0` /
  `CARGO_INCREMENTAL=0` は、Tauri と rumdl（tikv-jemalloc-sys）のビルドが
  GitHub runner のディスクを使い切ることへの対策である（共通ワークフローにも
  同趣旨の「ディスクを空ける」ステップが存在する）。ローカルの `verify.sh` へ
  全局適用する理由はなく、本次でも適用しない。
- `scripts/verify.sh:268` は `trunk build --offline --dist "$DIST_DIR"` であり、
  `--locked` が無かった。この状態では `Cargo.lock` が黙って更新され得る
  fail-open が残る。
- trunk 0.22.1 は `trunk build --release` で本番ビルドへ切り替える設計で、
  既定は dev プロファイル（`--profile` オプションは持たない）。
  `trunk build --help` は `--locked` を "Require Cargo.lock is up to date" と定義する。

## 決定

- `scripts/verify.sh:268` の trunk 行へ `--locked` を付与し、
  `trunk build --offline --locked --dist "$DIST_DIR"` とする。
  既存の `RUSTFLAGS="--remap-path-prefix=$home=~"` と `--offline` は維持する。
- `.github/workflows/ci.yml` と `scripts/ci-wasm-test.sh` は変更しない。
  監査の結果、wasm ゲートは二重実行ではなく、追加の修正が必要ないためである。
- `CARGO_PROFILE_*` / `CARGO_INCREMENTAL` は CI 側のままとし、ローカルへ移さない。

## 選択肢と根拠

| 選択肢 | 判断 | 根拠 |
| --- | --- | --- |
| `wasm_targets` を削除 | 却下 | `rustup target add` は `extra_gates` の wasm check と wasm-pack test の前提。二重実行ではないため削除する根拠がない。 |
| `--frozen` へ統一 | 未決定 | `--frozen` は `--locked` と `--offline` を合わせた意味で他ゲートへも波及する。既存の `--offline` を維持したまま統一範囲を別途判断する。 |
| `--locked` のみを付与（採用） | 採用 | 既存 `--offline` を維持しつつ `Cargo.lock` の更新を失敗させ、差分を 1 フラグに限定できる。 |
| env をローカル `verify.sh` へ全局適用 | 却下 | runner のディスク制約への対策であり、ローカルでは必要ない。trunk 行へスコープする案も未決定（後述）。 |

## 根拠（出典）

- 共通ワークフローの実ファイル:
  [rust-gates.yml@v1](https://raw.githubusercontent.com/Seduce0068/workflows/v1/.github/workflows/rust-gates.yml)
  - `wasm_targets` の説明は「rustup target add する対象（カンマ区切り、任意）」で、
    対応ステップの本体は `rustup target add "$target"` のみ。
  - `extra_gates` の説明は「固定ゲートの後に実行する追加コマンド（改行区切り、任意）」で、
    実際の wasm 検査はこちらに置かれている。
  - 呼び出し側の参照はタグ `v1` にピン留め（共通指示 5）。
- trunk 0.22.1 の CLI 仕様（ローカルの `trunk --version` / `trunk build --help` で確認）:
  `--locked` は "Require Cargo.lock is up to date"。`--release` が明示オプトインで、
  既定は dev プロファイル。
- 本次の変更点は `scripts/verify.sh:268`。監査依頼に 587 行目という記載があったが、
  実ファイルの trunk build 行は 268 行目である。

## 検証（Runtime Proof）

- 親エージェントの検証: `bash -n scripts/verify.sh` OK、`./scripts/verify.sh` 全ゲート rc=0、
  E2E **254/0**、`Cargo.lock` 差分なし。
- 文書エージェントの再検証（2026-11-19）: `bash -n scripts/verify.sh` 再実行 OK、
  `npx markdownlint-cli2 'docs/**/*.md' README.md todo.md` は 0 issues。
- 再現コマンド（Kubuntu / リポジトリルート）:

```bash
bash -n scripts/verify.sh
grep -n 'trunk build' scripts/verify.sh
npx --yes markdownlint-cli2 'docs/**/*.md' README.md todo.md
```

## 未決定

### 本番プロファイル（--release vs dev）

`verify.sh` の trunk build は既定 dev のまま。配布物を release でビルドするかは、
配布サイズ・デバッグ情報・署名方針と合わせて別途判断する。

### env の trunk 行へのスコープ

CI の `CARGO_PROFILE_*` を `verify.sh` の trunk 行だけへ適用する案は採用していない。
runner の disk 対策をローカルへ持ち込む必要性が現時点で確認できていないため。

### CI での trunk build / E2E の実行

`ci.yml:6` のコメントどおり trunk build + E2E は `verify.sh` 側のゲート。
共通ワークフロー側へ移すかは billing 解消後の実 CI 検証と併せて判断する。

### --frozen の統一

`--locked` 単独を採用したが、`--frozen` への統一は未決定のまま残す。

### 実 CI の検証

billing 問題で CI が実行できておらず、ローカル代替検証のみである。
解消後に実 CI で再確認する（TODO の CI-H01 として低優先度で継続）。

## 参照

- [.github/workflows/ci.yml](../../.github/workflows/ci.yml)
- [scripts/ci-wasm-test.sh](../../scripts/ci-wasm-test.sh)
- [scripts/verify.sh](../../scripts/verify.sh)
- [ADR-0196: wasm-pack test の CI ゲート化](0196-ci-wasm-test-gate.md)
- [ADR-0234: ドキュメント運用の一本化](0234-documentation-operations.md)
- [GOAL.md](../../GOAL.md)
