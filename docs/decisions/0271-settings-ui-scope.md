---
type: Decision
title: "ADR-0271: 設定保存 UI 接続スコープの設計受容"
description: "IPC 契約の完了と DTO / a11y / E2E の未実装を切り分け、UI 接続を完全な ConfigPatch DTO 定義後に留める決定。"
tags:
  - ADR
  - config
  - ui
  - ipc
generated:
  by: dsh
  at: "2026-11-20T05:33:26+09:00"
status: accepted
---

## 背景

R6 A05 の設定保存 UI 接続について、IPC 契約側と UI 側の進捗を切り分けて記録する
（swarm-coding / design による設計受容）。

- ADR-0270 で `withGlobalTauri: true` と
  `window.__TAURI__.core.invoke`（末尾 `__` 付き）の経路を確定し、
  ADR-0269 で `ConfigPatch.expected` による UI 起点 CAS と
  `CommandError { code, message }` による構造化エラーを実装した。
- 一方、R1 監査と現行コードの照合では、web 側の `SettingsView`
  （`web/src/lib.rs:4024-4090`）は読み取り専用 DTO のままで、
  desktop 側の `ConfigPatch`（`desktop/src/commands/config.rs:284-297`）へ
  Serialize できない。不足は `expected` を除く公開 DTO フィールド 23 項目に上る。
- storage の allow-list DTO（`StorageConfigView` / `LocalRootView` /
  `GoogleDriveConfigView`）は Rust 側が権威で、`local_roots[].path` と
  `recent_paths`、secret 値は JS へ開示しない。web 側に DTO を定義しても
  この契約を複製・緩和してはならない。
- a11y（`label` の `for` / `id`、`aria-invalid`、`aria-describedby`、
  操作結果の告知）、`CommandError.message` の英語固定文言を日本語 status へ
  変換する設計、Tauri mock E2E の成功経路、実 WebView 往路は未実装・未検証。
- 設計時の仮名 `ConfigUpdateRequest.expected` / `window.__TAURI.core.invoke` と
  実装名が食い違う命名ドリフトが残る（ADR-0253 に履歴として残存）。
  実装では `ConfigPatch.expected` / `__TAURI__` が正（ADR-0269 / 0270）。

本次は設計のみであり、web / desktop / E2E / Cargo.* は変更しない。

## 現状の切り分け

| 領域 | 状態 | 根拠 |
| --- | --- | --- |
| withGlobalTauri + `__TAURI__.core.invoke` 経路 | 完了 | ADR-0270 |
| UI 起点 CAS（`ConfigPatch.expected`）と構造化エラー | 完了 | ADR-0269 |
| CSP 整合性の静的ゲート | 完了 | ADR-0268 |
| DTO 往復（web `SettingsView` → `ConfigPatch`） | 未実装（23 フィールド不足 + `expected`） | GOAL.md / 本 ADR |
| storage allow-list の web 側 DTO 複製回避 | 未決定 | ADR-0102 / 0103 / 0186 / 本 ADR |
| a11y（`for` / `id` / `aria-invalid` / `aria-describedby` / status） | 未実装 | ADR-0253 実装前提 4 |
| `CommandError.code` → 日本語 status 変換 | 未実装 | ADR-0226 / 0269 |
| Tauri mock E2E 成功経路（+1） | 未実装（DTO 往復が先） | ADR-0253 実装前提 5 |
| 実 Tauri WebView の往路・復路・表示検証 | 未検証 | ADR-0264 / 0270 |
| 外部 JS 利用者の有無 | 未決定 | ADR-0270 |

## DTO 要件

R1 監査（GOAL.md 契約）は、web 側 `SettingsView` が `ConfigPatch` へ
Serialize できず 23 フィールド不足と記録した。allow-list View の差分を
現行コードから分解すると次となる（CAS 用 `expected` は除外）。

- `general.check_updates`（1 項目）
- `ai.openai.base_url` / `ai.openai.organization` /
  `ai.ollama.base_url` / `ai.ollama.keep_alive_minutes` /
  `ai.max_tokens_per_month` / `ai.streaming`（6 項目）
- `lint.enable_rules` / `lint.disable_rules`（2 項目）
- `storage.default_backend` / `storage.max_recent` / `storage.local_roots` /
  `storage.google_drive` と、その配下の
  `local_roots[].id` / `name` / `read_only` / `watch`、
  `google_drive.client_id` / `has_client_secret` / `has_refresh_token` /
  `root_folder_id` / `use_app_data_folder` / `scopes`（14 項目）

合計 1 + 6 + 2 + 14 = 23 項目。これに加えて `ConfigPatch.expected: u64` が
必須で、serde の default は禁止（ADR-0269）。

なお `ConfigPatch` は `EditorConfig` / `ReaderConfig` を直接保持するため、
web 側 `SettingsEditorView` / `SettingsReaderView` にも DTO 外フィールド
（`font_family` / `tab_size` / `auto_save_delay` / `column_width` /
`hyphenation` / `vertical_writing` など）が残る。完全な DTO を定義する段階で、
上記 23 項目と併せてフィールド単位の照合を確定する。

storage については、web 側 DTO を定義しても Rust 側の allow-list を権威とし、
`local_roots[].path` / `recent_paths` と secret 値（API key / client secret /
refresh token / `extra_headers`）を開示しない。存在フラグ `has_*` は
デシリアライズのために受け取っても更新時は無視し、現在値を保持する
（ADR-0102 / 0103 / 0186 / 0269）。

## 決定

- 本次はコード変更なしで設計受容とする。
- UI 接続は、web 側に `ConfigPatch` と往復可能な完全な DTO（本 ADR では
  仮に `SettingsPatch` と呼ぶ）を定義し、allow-list / 秘密 / パス非開示契約を
  Rust 側 DTO と同一にしてから行う。
- browser 単体の読み取り専用 / fail-closed 契約（ADR-0193 / 0253）を維持する。
- E2E check 名・件数（255）は不変とし、本次で +1 しない。成功経路 +1 は
  DTO 往復が完成した後の実装段階で追加する。
- 実 Tauri WebView 検証と外部 JS 利用者の確認は本次の範囲外とする。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 今すぐ UI 接続 | 採らない。web 側 DTO が 23 フィールド不足で `ConfigPatch` へ Serialize できず、隠しフィールドを既定値で上書きするリスクがある。英語固定文言の status 変換と a11y 契約も未定義。 |
| 設計受容（採用） | DTO 往復・allow-list・a11y・E2E 成功経路を先に設計し、段階ごとに検証できる。コード変更なしで fail-closed と秘密保護を維持する。 |
| 読み取り表示も廃止 | 採らない。ADR-0128 / 0193 の読み取り表示契約を後退させ、desktop の現状把握手段を失う。 |

根拠とする ADR と要件:

- ADR-0128 / 0193: 設定フォームの読み取り表示と browser の読み取り専用
  fail-closed。後退させず維持する。
- ADR-0211: 3 設定レイヤーの権威・寿命分離。web 側 DTO を単一の真実へ
  統合せず、IPC DTO 境界を守る。
- ADR-0226: status 文言の日本語統一と redaction。
  `CommandError.code` → 日本語 status 変換の根拠。
- ADR-0253: desktop 保存 UI の段階設計と実装前提 1〜6。
- ADR-0264 / 0270: IPC 経路の設計受容と確定。
- ADR-0269: UI 起点 CAS と構造化エラーの完了。
- DTO 要件: 上記 23 項目と storage allow-list の非開示契約。

## 影響

- 本次は文書のみ。web / desktop / E2E / Cargo.* は未変更で、
  E2E 255 件・check 名は不変。
- ADR-0253 に命名ドリフトを追記し、実装名を `ConfigPatch.expected` と
  `__TAURI__` として明示する（旧名は履歴として保持）。
- `todo.md` の A05 を「IPC 契約完了・UI 接続は DTO 設計後」へ更新する。
- decisions / docs 各 index（271 件）、README、log を追従する。

## 未決定

- DTO 形: 新規 `SettingsPatch`（partial DTO）とするか、`SettingsView` を
  拡張して完全な `ConfigPatch` 往復を担わせるか。不足 23 項目の数え方と
  editor / reader の DTO 外フィールドの扱いもここで確定する。
- storage の `path` / secret 非開示契約: `local_roots[].path` / `recent_paths`
  と `has_*` フラグ以外の秘密を web 側 DTO へ含めない具体的な型と検証手順。
- a11y: `aria-invalid` / `aria-describedby` の適用対象と、エラー告知の
  `role="status"` と既存 status の共存方法。
- `CommandError.code` → 日本語 status 変換:
  `CONFIG_CONFLICT` / `CONFIG_VALIDATION` / `CONFIG_STORAGE` /
  `CONFIG_INTERNAL` の文言表と redaction 契約。
- E2E +1 の範囲: DTO 往復完了後に追加する Tauri mock 成功経路の
  check 名と期待総数の更新手順（現状は 255 維持）。
- 実 Tauri WebView での往路・復路・表示・a11y 検証。
- 外部 JS 利用者の有無（破壊的変更の扱い）。

## 検証

- 文書形式: OKF `type: Decision`、`##` からの見出し階段、
  RFC 3339 `+09:00` のタイムスタンプ。
- `npx markdownlint-cli2 'docs/**/*.md' README.md todo.md`: 0 issues。
- コード・E2E は未変更。check 名・総数（255）は不変で、
  `./scripts/verify.sh` の最終確認は親セッションで行う。
