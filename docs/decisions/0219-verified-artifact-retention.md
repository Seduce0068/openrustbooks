---
type: Decision
title: "ADR-0219: 検証済み配布物のローカル保持"
description: "verify.sh --dist で全ゲート成功時のみ dist をリポジトリ外へ保持し、manifest で完全性を記録する決定。"
tags:
  - ADR
  - verification
  - distribution
  - recovery
generated:
  by: dsh
  at: "2026-11-19T14:10:45+09:00"
status: accepted
---

## 背景

`scripts/verify.sh` は `trunk build --offline` を `mktemp -d` へ出力し、EXIT trap で
毎回削除していた。そのため全ゲート成功後も検証済み配布物が残らず、
デプロイ／復旧の糸口が無い（R6 TD-M16）。
`web/dist` は git 管理外で開発パス残存の可能性があり、「検証済み」とは言えない。
CI は共通ワークフロー `Seduce0068/workflows/rust-gates.yml@v1` へ委譲しているが、
このワークフローは artifact upload 入力を持たず、billing issue も未解消である。
本 ADR は、共通ワークフローを変更せずローカルで完結する即時対応を記録する。

## 決定

- `scripts/verify.sh` に `--dist <dir>` を追加し、**全ゲート成功時のみ**検証済み dist を
  保持する。ゲート失敗時はコピーしない。
- 保持先は `<dir>/<short-sha>-<UTC>/`（UTC の `YYYYMMDDTHHMMSSZ`）。同名は上書きせず
  `-1`〜`-99` の連番を付与する。
- `manifest.txt` に commit sha、verified at（UTC）、rustc、trunk、
  e2e expected / e2e result、source dist、dist ファイルの SHA256（現状 5 行）を記録する。
- 一時 dist は常に trap で削除する。保持物はステージング経由で `mv -n -T` により
  原子的に配置し、中途半端な成果物を残さない。
- `DIST_ROOT` は絶対パス必須とし、`readlink -m` で正規化したうえでリポジトリ配下を拒否する。
- E2E 結果行や trunk バージョンを抽出できない場合は `unknown` を記録せず hard fail とする。
  成功風の数値は書かない。
- 引数なしの既定動作は不変（一時 dist を削除し、何も残さない）。
- 再利用・復旧の手順は `docs/runbook-verified-build.md` に集約する。

## 根拠

- R6 TD-M16: 検証成功後に配布物が残らず、復旧の糸口が無いという本件的な対処。
- 共通 `rust-gates.yml@v1` に upload 入力が無い。GOAL.md の制約で共通ワークフローは変更しない。
- billing issue が未解消で、実 CI における upload の検証ができない。
- `web/dist` は未検証の可能性があり、保持物の権威として使えない。
- 67 MB の wasm を git 追跡せず（`.gitignore` 維持）、リポジトリ外の絶対パスへ保持する。
- manifest の SHA256 は再実行なしで取得した既存情報から生成し、抽出失敗は成功扱いにしない。

## 影響

- 新規依存・外部送信・破壊的操作は追加しない。共通ワークフローは不変。
- 引数なしの `verify.sh` は従来どおり rc=0 で、成果物を残さない。
- E2E 253 件を維持する。git 管理対象は増えない。
- SHA256 は完全性のみを提供し、署名による真正性は未提供（未決定として継続）。

## 検証（Runtime Proof）

- 実測: `./scripts/verify.sh`（引数なし）rc=0、`./scripts/verify.sh --dist <dir>` rc=0。
- 実測: manifest の e2e expected は 253、e2e result は `passed: 253, failed: 0`。
- 実測: SHA256 は 5 行（favicon.svg / index.html / 生成 JS / `_bg.wasm` / `.trunk-lock`）。
- 本セッションで保持物を再確認: `sha256sum -c manifest.txt` は
  `WARNING: 10 lines are improperly formatted`（ヘッダ行由来）を出し、
  `grep -E '^[0-9a-f]{64} ' manifest.txt | sha256sum -c -` は
  manifest に記録された全ファイル（現状 5 行）が OK。
- 文書のみの変更のため compile / test は省略。verify.sh の実測は親エージェントの結果を記録。
- markdownlint-cli2（`/tmp/mdlint.jsonc`）: 全対象 0 issues。

## 未決定

- 保存先の固定場所と保持期間／ローテーション（現状は手動管理、`/tmp` は非推奨）。
  prune（古い世代の削除）の実施主体と、運用の責任者／期限も未定義。
- 本番プロファイル（`--release` / `--debug`）。現状は `trunk build` の既定プロファイル。
- `trunk build` への `--locked` 付与（CI とローカルの整合性）。
  wasm-opt は未対応（`web/Trunk.toml` に設定なし、PATH にも無し、ADR-0088 参照）。
- 共通ワークフローでの artifact upload（v1 タグの付け替え vs v2、self-test での検証）。
- billing 解消後の実 CI での upload 検証。責任者と期限は未定。
- 署名／真正性（SHA256 は完全性のみを保証する）。
- Release 専用ワークフロー（`contents: write` + 署名 secrets）との境界。
  チェックサム形式は SHA256 のみで真正性は無い（署名未提供）。
- 「検証済み」の定義（E2E 253 件充足の扱い、cargo audit warning baseline の扱い）。

## 参照

- [GOAL.md](../../GOAL.md) — 本件の契約
- [Runbook: 検証済み配布物の取得・保持・復旧](../runbook-verified-build.md)
- [scripts/verify.sh](../../scripts/verify.sh) — 実装
- [ADR-0214: MSRV 1.95 の宣言](0214-msrv.md)
- [todo.md](../../todo.md) — TD-M16 の完了記録
