---
type: Runbook
title: "リリース運用: 導入・更新・ロールバックと既知問題"
description: "公開 installer / updater / 署名成果物が無い現状を開示し、検証済み Web 配布物の導入・更新・ロールバック手順を定義する。"
tags:
  - runbook
  - release
  - distribution
  - disclosure
generated:
  by: dsh (release-runbook-writer)
  at: "2026-11-20T00:26:18+09:00"
---

## 概要

本 Runbook は、R6 R3-release#4,#5 の運用文書として現状を正直に開示し、
検証済み配布物を使った導入・更新・ロールバックの手順を定義する。
判断の経緯は [ADR-0254](decisions/0254-release-operations-disclosure.md) を参照。

### 現状の開示

- **公開 installer は無い**。Windows / macOS / Linux 向けの `.msi` / `.dmg` /
  `.AppImage` 等は配布していない。Tauri CLI は開発用であり、公開用成果物の
  ビルド・署名・ホスティングは行わない。
- **updater は未配線**。`tauri_plugin_updater` は登録すらせず、
  `UpdaterExt::check` / `.check()` の呼び出しも無い（ADR-0222 / ADR-0277 / ADR-0279 /
  ADR-0282）。設定画面の表示は固定文言「更新確認: 未配線」で、
  `check_updates` は死設定である。pubkey / endpoint / 署名 workflow は未実装で、
  `scripts/check-updater-honesty.sh` が plugin 登録・`.check()`・pubkey / endpoint の
  再混入を fail-closed で拒否する。
- **署名済み成果物は無い**。minisign 検証は fail-closed だが、署名秘密鍵・公開鍵が
  存在しないため署名付きリリースは作っていない。
- **公開 endpoint は無い**。更新用の HTTPS URL は未定義で、
  desktop の CSP `connect-src` にも更新用 endpoint は含まれない。
- 利用できるのは `verify.sh --dist` でローカルに保持した検証済み Web 配布物を
  HTTP で serve する経路のみである（ADR-0219）。

## 前提

- `scripts/verify.sh` の全ゲートが成功していること。
  ゲートは fmt / check / test / clippy / wasm / dependency-hygiene /
  audit-db-freshness / audit / `trunk build` / build-desktop / check-frontend-dist /
  パス検査 / secrets / E2E / WDIO を含む。E2E の期待総数は現行契約
  （コード出典 `Checker(expected=273)`、[GOAL.md](../GOAL.md) のモバイル記録では
  273 件）に従う。
  [runbook-verified-build.md](runbook-verified-build.md) に残る 253 件は旧実測例であり、
  現行の期待値ではない。
- [docs/runbook-verified-build.md](runbook-verified-build.md) の手順で保持した
  検証済み dist が存在し、`manifest.txt` と SHA256 で完全性を確認できること。
  未検証の `web/dist`（開発ビルドの残存）は配布物として使わない。
- 保存先はリポジトリ外の絶対パス（推奨:
  `/home/user/.local/share/openrustbooks-verified/`）。`/tmp` は再起動で
  消えるため恒久保持先として使わない。
- シークレットゲートは Betterleaks v1.10.1 を使う。ローカルは pre-commit
  （`.pre-commit-config.yaml` の rev `v1.10.1`）を主軸とし、`verify.sh` と CI の
  `extra_gates` が最終ゲートとして fail-closed で検査する。未導入時は
  `bash scripts/install-betterleaks.sh [bin-dir]` で checksums.txt 検証付きに導入できる
  （CLI は `betterleaks git <repo> --config .betterleaks.toml --redact --verbose`、Gitleaks の `detect --source` ではない）。
- Tauri CLI（`cargo install tauri-cli`）は desktop のローカル開発専用である。
  公開 installer をビルド・署名・配布するためのものではない。

## 導入

### Web 版

検証済み dist を HTTP で serve し、ブラウザから開く。手順の詳細は
[runbook-verified-build.md](runbook-verified-build.md) の「配布物の serve」に従う。

```bash
# Kubuntu: 保持先で実行（<short-sha>-<UTC> は実値へ置換）
DIST=/home/user/.local/share/openrustbooks-verified/<short-sha>-<UTC>
echo "=== SHA256 検証 ==="
(cd "$DIST" && grep -E '^[0-9a-f]{64} ' manifest.txt | sha256sum -c -)
echo "=== serve（Ctrl-C で停止） ==="
python3 -m http.server --directory "$DIST" 8000
```

ブラウザで `http://localhost:8000/` を開き、Web 版リーダーが表示されることを確認する。
serve はローカル検証用であり、公開 Web ホスティングではない。

### desktop 版

desktop はローカル source からビルドして起動する。リポジトリを clone し、
Rust 1.95 以上（MSRV、ADR-0214）・trunk・`wasm32-unknown-unknown` ターゲット・
Tauri CLI を用意してから `cargo tauri dev` を実行する。
`beforeDevCommand` が `trunk serve` を起動する（[README.md](../README.md) 参照）。

**公開 installer は無いため、利用者が自分で source からビルドする必要がある。**
ビルド済みバイナリの配布、署名、自動インストールは提供しない。

## 更新

自動更新は無い。`tauri_plugin_updater` は登録すらせず、`check_updates` の値は
true / false のどちらでも動作へ接続されず、設定表示は「更新確認: 未配線」のまま
変わらない（ADR-0222 / ADR-0277）。公開 endpoint と署名 manifest も無いため、
updater 経由の更新はできない。

更新は次の手動手順で行う。

1. 現在適用している revision と short-sha を `manifest.txt` から記録する。
2. 対象 revision へ切り替え、`verify.sh` の全ゲートを再実行する。
3. `verify.sh --dist` で新しい検証済み dist を保持し、manifest と SHA256 を検証する。
4. 新しい世代を serve し、Web 版リーダーの表示と主要操作を確認する。
5. 問題が無ければ旧世代は手動で残すか整理する（保持期間とローテーションは未決定）。

```bash
# Kubuntu: リポジトリルートで実行（<revision> は実値へ置換）
cd /home/user/ドキュメント/作業用/openrustbooks
echo "=== 現在の revision ==="
git rev-parse --short HEAD
echo "=== 全ゲート再検証と再保持 ==="
./scripts/verify.sh --dist /home/user/.local/share/openrustbooks-verified/
```

git 操作は適用者が行う。本 Runbook は手順の定義のみで、
自動チェック・自動ダウンロード・自動インストールを要求しない。

## ロールバック

自動 rollback は無い。更新後の不具合では、保持済み dist の `manifest.txt` に
記録された commit sha とディレクトリ名の short-sha を使って直前の revision を
特定し、その世代へ手動で戻す。

1. 保持先の世代一覧を新しい順に確認する。
2. 戻す世代の `manifest.txt` の commit sha / verified at / E2E 結果を確認する。
3. SHA256 を検証する。不一致または欠落があればその世代は使わず、別世代へ切り替える。
4. 現在の serve を停止し、選択した世代を serve し直す。
5. 全世代が検証失敗なら `verify.sh --dist` で新しい検証済み dist を生成する。

```bash
# Kubuntu: 保持先で実行（<short-sha>-<UTC> は戻す世代の実値へ置換）
echo "=== 保持世代一覧 ==="
ls -1 /home/user/.local/share/openrustbooks-verified/
DIST=/home/user/.local/share/openrustbooks-verified/<short-sha>-<UTC>
echo "=== manifest 確認 ==="
cat "$DIST/manifest.txt"
echo "=== SHA256 検証 ==="
(cd "$DIST" && grep -E '^[0-9a-f]{64} ' manifest.txt | sha256sum -c -)
echo "=== 切り替えて serve（Ctrl-C で停止） ==="
python3 -m http.server --directory "$DIST" 8000
```

### config / localStorage のバックアップ

ロールバック前に設定と読書データをエクスポートし、戻す側へインポートする。
対象は設定、読書位置、移動履歴、エディタ下書きを含むバックアップ JSON である
（ADR-0022 / 0026 / 0030 / 0180）。
インポートは現在値を保護する契約（ADR-0157）に従うため、
上書き前にエクスポートを取得しておくこと。
config.toml のファイルバックアップも適用前に取得する。

## 識別子変更に伴う設定移行

- `desktop/tauri.conf.json` の `identifier` / `bundle.publisher` を個人識別子から
  プロダクト名ベースへ変更した。旧ビルドの Tauri データ／バンドルディレクトリは
  旧識別子のままであり、新ビルドは別ディレクトリを作成する。
- アプリ設定 (`config.toml`) とカスタムプロンプト (`prompts/`) の解決先は
  `ProjectDirs` の qualifier / organization に依存する。これも個人識別子から
  プロダクト名ベース（`com.openrustbooks`）へ変更した。OS によっては旧ディレクトリと
  別の場所を参照するため、以下の手順で引き継ぐ。
  1. 旧ディレクトリの `config.toml` と `prompts/` を特定し、バックアップを取得する。
  2. 新ビルドを一度起動して新ディレクトリを生成する。
  3. 旧 `config.toml` と `prompts/` を新ディレクトリへコピーする。Unix では
     `config.toml` のパーミッション 0600 を維持する。
  4. 新ビルドで設定を読み込み、上記のエクスポート／インポート手順で内容を検証する。
  5. 旧ディレクトリは即時削除せず、バックアップとして残す。
  自動移行は行わない。Linux は `ProjectDirs` が application 名のみをパスに使うため、
  設定パス自体は変わらない場合がある。

## 既知問題

| 既知問題 | 現状 | 根拠 |
| --- | --- | --- |
| updater 未配線 | plugin 未登録。`.check()` / 公開鍵 / endpoint / 署名 workflow は未実装で、`check-updater-honesty.sh` が再混入を拒否 | ADR-0222 / ADR-0277 / ADR-0279 / ADR-0282 |
| browser AI 無効 | ブラウザの AI API は明示エラー。native のみ提供、既定無効 | ADR-0232 / ADR-0156 |
| 設定が読み取り専用 | browser は fail-closed readonly。desktop は mock 接続（273/0）＋実 WebView 検証済み。WDIO 独立ゲート運用中。mock 273/0 は実 WebView の代替ではない | ADR-0193 / ADR-0253 / ADR-0272 / ADR-0273 / ADR-0282 / ADR-0283 |
| 実機 5 ブラウザ未検証 | Chrome / Edge / Brave / Firefox / Safari の実機検証は未実施 | ADR-0232 |
| WDIO は dev server のみ | 実 WebView E2E は trunk dev server 経由。バンドル済みバイナリの `/settings` は SPA fallback が無く `asset not found` | ADR-0283 / docs/runbook-webdriver.md |
| CI billing | billing 未解消のため実 CI の検証と artifact upload は未実施 | ADR-0219 / ADR-0237 |
| OS キーチェーン未配線 | API キー等の平文保存からの移行は未決定（マスク済み、設計受容） | ADR-0098 / ADR-0261 |

desktop 設定保存の前提別状態（一部完了 / 方針のみ / 完了 / 未検証 / 環境ブロッカー）は
[ADR-0274](decisions/0274-a05-traceability-reconciliation.md) を参照。これは当時の
未検証状態の記録であり、現在は ADR-0282 / ADR-0283 で実 WebView 検証済みである。
Tauri mock E2E 273/0 は JS / IPC 契約レベルであり、実 WebView の代替ではない（ADR-0273）。
実 WebView では検証時点の Tauri CLI 2.12.5 で settings / get_config / update_config_patch /
config.toml 永続化を検証済み（ADR-0282）。WebdriverIO による独立 E2E ゲートも
`verify.sh` の末尾で実行する（ADR-0283、
[WebdriverIO 実 WebView E2E](runbook-webdriver.md)）。

自動更新、自動 rollback、公開 installer、署名済み成果物は上記のとおり提供しない。
公開 installer / 署名成果物の導入判断は未決定として
[ADR-0254](decisions/0254-release-operations-disclosure.md) に集約する。
updater は未配線のまま固定表示と `check-updater-honesty.sh` の静的ゲートで維持する
（ADR-0222 / ADR-0277 / ADR-0279）。

## 署名秘密鍵の取扱い

- 署名秘密鍵と release secrets はリポジトリ外（運用者の管理下）または
  将来の Release workflow の CI secrets に置く。リポジトリへコミットしない。
- `.gitignore` で minisign 秘密鍵・署名 material のパターンを拒否する。
  本件の `.gitignore` 変更は R6 で親が完了済み（署名 material のみ、公開鍵は対象外）。
- 公開鍵の設定（`plugins.updater.pubkey`）は updater 実装後に設定する。
  現状は pubkey / endpoint / 署名 workflow が未実装で、`plugins.updater` ブロック自体が
  存在しない（ADR-0282）。`scripts/check-updater-honesty.sh` が再混入を拒否する。
- `require_signed_version` の有効化時期も未決定である（ADR-0222）。
- manifest の SHA256 は完全性のみを保証し、署名による真正性は保証しない。

## 参照

- [ADR-0254: リリース運用の文書先行開示](decisions/0254-release-operations-disclosure.md)
- [ADR-0222: updater 未配線の設計受容](decisions/0222-updater-deferred.md)
- [ADR-0277: updater 死設定の fail-closed 復元](decisions/0277-updater-honesty-restoration.md)
- [ADR-0279: updater チェック呼び出しの検出パターン拡張](decisions/0279-updater-check-pattern.md)
- [ADR-0282: 実 WebView 起動と設定保存の Runtime Proof](decisions/0282-real-webview-runtime-proof.md)
- [ADR-0283: WebdriverIO による実 WebView E2E の導入](decisions/0283-webdriverio-adoption.md)
- [ADR-0219: 検証済み配布物のローカル保持](decisions/0219-verified-artifact-retention.md)
- [Runbook: 検証済み配布物の取得・保持・復旧](runbook-verified-build.md)
- [GOAL.md](../GOAL.md) — 本件の契約
- [docs/index.md](index.md) — ドキュメント一覧
