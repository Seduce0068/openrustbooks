---
type: Decision
title: "ADR-0222: updater 未配線の設計受容"
description: "tauri_plugin_updater の初期化のみで更新チェックは未配線とし、設定表示を「未配線」へ正直化する決定を記録する。"
tags:
  - ADR
  - security
  - desktop
  - distribution
generated:
  by: dsh
  at: "2026-11-19T14:51:43+09:00"
status: accepted
---

## 背景

R6 TD-M21。`tauri_plugin_updater` は `desktop/src/main.rs:46` で初期化されるが、
更新チェックの呼び出し・公開鍵・endpoint・署名 secrets・Release workflow は未定義。
`check_updates`（既定 true）は設定画面に「有効」と表示されるだけで、
実際には更新の確認もダウンロードもインストールも行われない（死設定）。
公開鍵と署名鍵、配布 endpoint が無い状態でチェックだけを配線しても
minisign 検証で fail-closed となり動作しない。
本 ADR は GOAL.md の契約に基づき、未配線を設計受容し表示を正直化する判断を記録する。
ADR-0218（AI カスタムエンドポイント）と同型の判断である。

## 現状

- `desktop/src/main.rs:46`: `.plugin(tauri_plugin_updater::Builder::new().build())`
  のみ。初期化にとどまり、`UpdaterExt::check` 等の呼び出しは 0 件。
- `desktop/tauri.conf.json:49`: `createUpdaterArtifacts: true` のみ。
  `plugins.updater` セクション、pubkey、endpoints は存在しない。
- `desktop/tauri.conf.json:29` の CSP `connect-src` に更新用 endpoint は無い
  （`api.openai.com` / `localhost:11434` 等のみ）。
- `Cargo.lock:6730-6731`: tauri-plugin-updater 2.14.1。
  `Cargo.lock:3641` に minisign-verify があり、署名検証は fail-closed。
- `core/src/config.rs:42,57`: `check_updates: bool` は既定 true。
  `desktop/src/commands/config.rs:67,77,304,1237` で DTO として往復するが、
  動作へは接続されていない。
- `web/src/lib.rs:3999`: 設定表示は「更新確認: 未配線」（表示変更後の文言）。
- 署名秘密鍵、公開鍵の生成・配布、endpoint URL、Release workflow は未定義
  （調査時点の `todo.md:60,110-115`。完了移動後は `todo.md:132,108-113`）。

## 決定

- updater の未配線を**設計受容**（見送り）とする。
- `check_updates` の値に関わらず、設定画面の表示を「更新確認: 未配線」の固定文言へ変更する。
- updater の本番接続・自動チェック・自動インストールは実装しない。
- 設定ファイル・DTO・E2E は変更しない（E2E 253 件を維持）。
- 共通ワークフロー `Seduce0068/workflows/rust-gates.yml@v1` は変更しない。
- 将来実装する場合の前提条件と未決定を、本 ADR に集約する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 設計受容＋表示の正直化（採用） | 署名・配布・同意の契約が未定義で、接続しても安全に運用できない |
| 今すぐ `UpdaterExt::check` を配線 | pubkey / endpoint が無く fail-closed。UI の誤認も解消されない |
| `check_updates` を設定から削除 | config.toml / DTO の互換を壊し、移行方針も未定義 |
| 自動チェック・自動インストールを有効化 | 署名検証・明示的同意・ロールバック手順が未整備でリスクが高い |

## 将来の最小安全設計

実装する場合、同時に次を満たすものとする（今回は実装しない）。

### 署名と鍵

- minisign 検証を維持する（tauri-plugin-updater / minisign-verify）。
- `plugins.updater.pubkey` に公開鍵を設定する。
- `require_signed_version: true` とし、未署名バージョンを拒否する。
- `allow_downgrades: false` とし、ダウングレードを拒否する。
- `dangerous*` 系の検証無効化オプションは使用しない。
- 公開鍵はリポジトリ内の設定へ置き、秘密鍵はリポジトリ外（CI secrets）へ置く。
  秘密鍵はコミットせず `.gitignore` で拒否する
  （調査時点の `todo.md:115`、移動後は `todo.md:113`）。

### endpoint と CSP

- endpoint は HTTPS のみとし、host を固定する（userinfo 拒否、redirect は無効化または再検証）。
- CSP の `connect-src` に必要な endpoint のみ追加する
  （`desktop/tauri.conf.json:29`、必要なら Web 側も含む、ADR-0091 / 0130 を継続）。

### チェックと同意

- チェックは手動操作のみとし、起動時の自動チェックは行わない。
- download と install を分離し、install の直前に Rust 側のネイティブダイアログで
  明示的同意を要求する（ADR-0104 の拡張）。JS から `consent: bool` は受け取らない。
- 表示項目はバージョン・サイズ・release notes 等を候補とし、
  秘密情報や完全な署名値は画面に出さない。

### Release workflow

- Release 専用 workflow を設け、`contents: write` と署名 secrets で
  署名済み成果物と更新 manifest を生成する。
- 共通 `rust-gates.yml@v1` は変更しない（GOAL.md の制約、ADR-0219 の未決定を継続）。
- ホスティングと billing の責任者・期限を定義してから配布を有効化する。

## 根拠

- ADR-0218: 未接続機能を設計受容とし、現状契約と将来の最小安全設計を分離した同型判断。
- ADR-0219: 配布・billing が未決定で、Release workflow との境界も未定義。
- ADR-0216 / 0217: 死設定を削除せず「未配線」と表示する正直化の前例。
- ADR-0104: ネイティブ同意ダイアログの設計基盤（JS bool を信用しない）。
- ADR-0091 / 0130: CSP の権威と desktop / web の差。
- `Cargo.lock:6731` の 2.14.1 と `minisign-verify`: 署名が無い限り更新は fail-closed。
- `todo.md:60`（調査時点、完了移動後は `todo.md:132`）: TD-M21 の本件対処。
- `todo.md:110-115`（調査時点、移動後は `todo.md:108-113`）:
  公開鍵・署名・Release・鍵管理の残課題。
- GOAL.md: 本件の契約（本番接続・自動チェック・自動インストールは実装しない）。

## 影響

- 表示は「更新確認: 未配線」となり、`check_updates` の true / false は表示に影響しない。
- `check_updates` は死設定のまま config.toml に残る（未決定として継続）。
- 新規依存なし、unsafe なし、E2E 253 維持、共通ワークフロー不変。

## 検証（Runtime Proof）

- `desktop/src/main.rs:46`、`desktop/tauri.conf.json:29,49`、`core/src/config.rs:42,57`、
  `desktop/src/commands/config.rs` を読み、現状をコードで確認した。
- `*.rs` の grep で `UpdaterExt::check` 一致 0 件、`check_updates` は config / DTO のみを確認。
- `Cargo.lock:3641,6730-6731` と `THIRD_PARTY_NOTICES.md` で 2.14.1 と
  minisign-verify を確認。
- 文書のみの変更のため compile / test は省略。設定表示の変更は別途ゲート検証する。
- markdownlint-cli2（`/tmp/mdlint.jsonc`）: 全対象 0 issues。

## SettingsForm editable 化後の再確認（2026-11-20 追記）

- ADR-0272 の editable 化で固定表示が一時的に失われたが、ADR-0277 で復元済み。
- 現在は `check_updates` を DTO 往復のため保持しつつ、UI は「更新確認: 未配線」を常時表示し、保存時は現在値を保持する。
- 本 ADR の「設定ファイル・DTO・E2E は変更しない」「E2E 253 件維持」は決定時点の履歴。現行は ADR-0277 / E2E 256 件を参照。

## 未決定

- 公開鍵の生成・配布・ローテーションと失効（kill switch を含む）。
- endpoint URL / ホスティング、billing の責任者と期限。
- Release workflow の方式（共通 v1 のタグ付け替え vs 破壊的変更での v2）。
- `require_signed_version` を有効化する時期。
- 手動チェック / 自動チェックの別と、`check_updates` の配線または削除。
- 同意の粒度（更新ごと / バージョンごと / セッションごと）と表示項目。
- capabilities の permission、CSP の追加範囲、redirect の扱い。
- 更新失敗時のロールバック手順。
- mobile / wasm での updater の扱い（現状は配線対象外）。

## 参照

- [GOAL.md](../../GOAL.md) — 本件の契約
- [ADR-0218: AI カスタムエンドポイント opt-in の設計受容](0218-custom-ai-endpoint.md)
- [ADR-0219: 検証済み配布物のローカル保持](0219-verified-artifact-retention.md)
- [ADR-0104: AI 送信前のネイティブ同意ダイアログ](0104-send-consent-dialog.md)
- [todo.md](../../todo.md) — TD-M21 / R1-tauri#8 / R3-release#1〜5
