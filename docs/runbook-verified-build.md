---
type: Runbook
title: "検証済み配布物の取得・保持・復旧"
description: "verify.sh --dist で全ゲート成功時にのみ dist を保持し、manifest 検証と serve で復旧する手順。"
tags:
  - runbook
  - verified-build
  - verify
  - recovery
generated:
  by: dsh
  at: "2026-11-19T14:10:45+09:00"
---

## 概要

`scripts/verify.sh --dist <dir>` は、全検証ゲート成功時にのみ検証済みの Web 配布物を
`<dir>/<short-sha>-<UTC>/` へ保持し、`manifest.txt` を生成する。
既定（引数なし）の動作は従来どおりで、一時 dist は EXIT trap により毎回削除される。

本 Runbook は、保持した配布物の検証・配信・復旧の手順を定義する。
設計判断は [ADR-0219](decisions/0219-verified-artifact-retention.md) を参照。

## 前提

- Rust 1.95 以上（MSRV、ADR-0214）と `cargo-audit`。未インストール時は audit ゲートが
  fail-closed で失敗する。
- `gitleaks`（シークレットスキャン）。未インストール時は `check-secrets.sh` が
  fail-closed で失敗する。ローカルは `pre-commit install` で pre-commit を主軸にし、
  `verify.sh` と CI の `extra_gates` を最終ゲートとして併設する。
  pre-commit の gitleaks hook は Go 製のため、初回構築時には Go toolchain が必要。
- `trunk` 0.22.1。E2E は Python 3.10 以降（`websockets`）と Chrome / Chromium を使う。
- `wasm-bindgen` が PATH にあること。`verify.sh` は必要に応じて
  `$HOME/.cache/trunk/wasm-bindgen-0.2.129` を PATH へ追加する。
- 保存先は**リポジトリ外の絶対パス**。`--dist` は相対パスとリポジトリ配下を拒否する。
  推奨先は `/home/koppe/.local/share/openrustbooks-verified/`。
  `/tmp` は再起動で消えるため恒久保持先としては避ける。
- `trunk build --offline` が成功するローカルキャッシュが揃っていること（新規依存を取得しない）。

## 手順

### 検証済み dist の生成

Kubuntu 上でリポジトリルートへ移動し、実際の絶対パスを指定して実行する。

```bash
# Kubuntu: リポジトリルートで実行
cd /home/koppe/ドキュメント/作業用/openrustbooks
echo "=== verify.sh --dist ==="
./scripts/verify.sh --dist /home/koppe/.local/share/openrustbooks-verified/
```

成功すると `===== 全検証成功 =====` の後、
`===== 検証済み dist を保持しました: .../2e730ab-20261119T050859Z =====` が出力される。
ディレクトリ名は `<short-sha>-<UTC>`（UTC の `YYYYMMDDTHHMMSSZ`）である。
同名ディレクトリが既存なら `-1`〜`-99` の連番を付け、既存物は上書きしない。

### manifest の確認

`manifest.txt` には commit sha、verified at（UTC）、rustc、trunk、
e2e expected / e2e result、browser executable / path / version、
static port / cdp port、source dist、および dist 内ファイルの SHA256
（現状 5 行）が記録される（ADR-0266）。
歴史的な実測例では `e2e expected : 253`、`e2e result   : passed: 253, failed: 0` だった。
現行の期待値は **256**（check 239 + equal 17、ADR-0272 の Tauri mock +1）で、
最新の実測は `./scripts/verify.sh` の Runtime Proof を参照する。

```bash
# Kubuntu: 保持先で実行（<short-sha>-<UTC> は実値へ置換）
DIST=/home/koppe/.local/share/openrustbooks-verified/<short-sha>-<UTC>
echo "=== manifest ==="
cat "$DIST/manifest.txt"
echo "=== SHA256 検証（ヘッダ行を除外） ==="
(cd "$DIST" && grep -E '^[0-9a-f]{64} ' manifest.txt | sha256sum -c -)
```

### SHA256 の検証

`manifest.txt` のヘッダ行は checksum 形式ではない。素の
`sha256sum -c manifest.txt` は実測で
`sha256sum: WARNING: 10 lines are improperly formatted` を出す。
ヘッダ行を除外するため `grep -E '^[0-9a-f]{64} ' manifest.txt | sha256sum -c -` を使う。
パスは `./` 始まりの相対パスなので、必ず配布ディレクトリ内で実行する。
manifest に記録された全ファイル（現状 5 行）が `OK` なら完全性は維持されている。
不一致や欠落があればその世代は使わない。
SHA256 は完全性のみを保証し、署名による真正性は保証しない。

### 配布物の serve

配布物は HTTP 経由で開く。`python3 -m http.server --directory <dir>` を使う
（ポート省略時は 8000）。確認後は `Ctrl-C` で停止する。

```bash
# Kubuntu: 任意のシェルで実行（<short-sha>-<UTC> は実値へ置換）
DIST=/home/koppe/.local/share/openrustbooks-verified/<short-sha>-<UTC>
python3 -m http.server --directory "$DIST" 8000
```

ブラウザで `http://localhost:8000/` を開き、Web 版リーダーが表示されることを確認する。

### 復旧手順

保持先から利用する世代を選び、SHA256 を検証してから serve する。
検証に失敗した場合は直近の別世代へ切り替える。すべて失敗する場合は
`verify.sh --dist` を再実行して新しい検証済み dist を生成する。

```bash
# Kubuntu: 保持先で実行（新しい順に並べ、<short-sha>-<UTC> は実値へ置換）
echo "=== 保持世代一覧 ==="
ls -1 /home/koppe/.local/share/openrustbooks-verified/
DIST=/home/koppe/.local/share/openrustbooks-verified/<short-sha>-<UTC>
echo "=== SHA256 検証 ==="
(cd "$DIST" && grep -E '^[0-9a-f]{64} ' manifest.txt | sha256sum -c -)
echo "=== serve（別シェルでも可、Ctrl-C で停止） ==="
python3 -m http.server --directory "$DIST" 8000
```

### 失敗時の扱い

- ゲート失敗（fmt / check / test / clippy / wasm / audit など）時は E2E と trunk build へ
  進まず rc=1 となる。配布物のコピーは行われない。
- E2E 結果行（`passed: N, failed: N`）や trunk バージョンを抽出できない場合は
  `unknown` とせず hard fail する。成功風の数値は記録しない。
- 保存先が相対パス、またはリポジトリ配下（`readlink -m` で正規化して判定）なら rc=2 で拒否する。
  判定は `scripts/check-distribution-path.sh` に集約し、CI の `extra_gates` からも同じ
  スクリプトを呼ぶ。同スクリプトは manifest の必須キー / `e2e expected=256` /
  `passed>=1` かつ `failed: 0` / SHA256 行形式も検査し、不合格なら保持を行わず
  hard fail する（期待件数の現行契約 256 と変更時の追従先は ADR 側で管理）。
  CI の無引数呼び出しでも自己検査として正負両方を実行する（恒真ゲートではない）。
- `web/index.html` に第三者フォント参照（`fonts.googleapis` / `fonts.gstatic` /
  `preconnect`）が含まれる場合は `check-no-external-font-refs.sh` が失敗する。
- `gitleaks` 未インストール時は `check-secrets.sh` が「install gitleaks」を表示して
  rc=127 で失敗する。検出時も gitleaks の終了コードを伝播して fail-closed とする。
  CI の共通 workflow には gitleaks 導入入力が無いため、`extra_gates` の末尾で
  存在を検査し、未導入ならそこで fail-closed とする。末尾へ置いたことで、
  未導入でも他の静的ゲート / wasm check / `ci-wasm-test.sh` は実行済みとなる。
- コピーはステージング（`*.tmp.<PID>`）経由で `mv -n -T` により原子的に配置する。
  中途失敗時は EXIT trap がステージングを削除し、不完全な成果物は残らない。
- 同名ディレクトリは上書きせず連番を探索する。`-99` まで使用中ならエラーとなる。
- 保持物に 67 MB の wasm が含まれても git には追加しない（`.gitignore` を維持）。

### 既知の制約

- 引数なしの `verify.sh` では何も残らない。保持したい場合は毎回 `--dist` を明示する。
- CI の共通ワークフロー `rust-gates.yml@v1` は artifact upload 入力を持たず、
  billing issue も未解消のため、保持はローカル実行に限られる。
  gitleaks の導入入力も無く、呼び出し側の `apt_packages` では Ubuntu 標準
  リポジトリに gitleaks が無いため、**現状の CI シークレットゲートは常時 fail
  （fail-closed）を受容している**。解消策は共通 workflow 側の導入に限らず、
  `extra_gates` 内でチェックサム付きバイナリを取得するか
  `go install github.com/gitleaks/gitleaks/v8@v8.28.0` で導入する選択肢もある。
  gitleaks ゲートは `extra_gates` の末尾（最終ゲート）に置いているため、
  未導入でも他の静的ゲート / wasm check / `ci-wasm-test.sh` は実行され、
  末尾で fail-closed となる。下記の billing 解消後チェックリストを参照。
- `web/dist` は git 管理外で未検証の開発パス残存の可能性がある。復旧には本手順の保持物を使う。
- 本番プロファイル（`--release` / `--debug`）は未決定（ADR-0219）。
  `trunk build --locked` は ADR-0237 で決定済みで、verify.sh / build-desktop.sh は
  `--offline --locked` を実行する（ADR-0289）。
- 保存先の固定場所と保持期間／ローテーションは未決定で、現在は手動管理である。
- SHA256 は完全性のみ。署名／真正性、Release 専用ワークフローとの境界は未決定（ADR-0219）。

## billing 解消後のチェックリスト

外部サービスの billing issue 解消後、以下を順に確認してから CI のゲートを更新する。
共通 workflow（`Seduce0068/workflows/rust-gates.yml@v1`）自体は勝手に変更しない。

- [ ] CI で gitleaks を利用可能にする。共通 workflow 側で導入するほか、
      `extra_gates` 内でチェックサム付きバイナリを取得する、または
      `go install github.com/gitleaks/gitleaks/v8@v8.28.0` で導入する選択肢がある。
      現状は未導入でも、gitleaks は `extra_gates` の末尾（最終ゲート）へ
      移しているため、他の静的ゲート / wasm check / `ci-wasm-test.sh` は
      すべて実行済みで、末尾の存在検査でのみ fail-closed となる。
      導入後は存在検査を通過して `check-secrets.sh` の実スキャンが実行され、
      シークレット未検出なら CI が green になる可能性がある。
      検出時は従来どおり fail-closed を維持する。
- [ ] CI で `trunk build` が成功することを確認する（現在は `verify.sh` 側のみ）。
- [ ] CI で E2E **256 件**（check 239 + equal 17、ADR-0272 の Tauri mock +1）と
      WDIO（`scripts/ci-wdio-test.sh`）が成功することを確認する。
- [ ] `manifest.txt` の `e2e expected` / `e2e result` が 256 / `failed: 0` であることを
      確認し、配布保持（ローカルの `verify.sh --dist`）が原子的に行われることを確認する。
- [ ] 実 CI での artifact upload を有効化し、保持物をダウンロードして
      SHA256 検証（`grep -E '^[0-9a-f]{64} ' manifest.txt | sha256sum -c -`）まで行う。
- [ ] `verify.sh` の全ゲートをローカルで再実行し、check 名
      （`no-external-font-refs` / `secrets` に加え、`--dist` 指定時の
      `check-distribution-path.sh` の受け入れ）と全ゲート成功を確認する。

## 参照

- [ADR-0219: 検証済み配布物のローカル保持](decisions/0219-verified-artifact-retention.md)
- [GOAL.md](../GOAL.md) — 本件の契約
- [scripts/verify.sh](../scripts/verify.sh) — 実装
- [check-distribution-path.sh](../scripts/check-distribution-path.sh)
- [check-no-external-font-refs.sh](../scripts/check-no-external-font-refs.sh)
- [check-secrets.sh](../scripts/check-secrets.sh)
- [docs/index.md](index.md) — ドキュメント一覧
