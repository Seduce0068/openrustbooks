---
type: Decision
title: "ADR-0261: OS キーチェーン移行の設計受容"
description: "OS キーチェーン移行を設計受容として記録し、ADR-0098 の未決定へ集約して今回の実装を見送る決定を記録する。"
tags:
  - ADR
  - security
  - config
  - keychain
  - deferred
generated:
  by: dsh
  at: "2026-11-20T02:45:37+09:00"
status: accepted
---

## 背景

ADR-0098 は API キー / `client_secret` / `refresh_token` の `config.toml`
平文保存を維持し、OS キーチェーン移行を「新規依存と設計判断が必要」として
見送っていた（R6 D05）。監査の結果、redaction / DTO 境界 / Unix 0600 は維持済みだが、
次が未解決であることが判明した。

- Wasm はキーチェーンの対象外（config 永続化自体が無い）。
- 平文 config.toml からキーチェーンへの移行手順が未定義。
- CI では秘密情報を使わないため実キーストアを検証できない。
- `core/src/config.rs` の doc コメント `encrypted` が実装と矛盾する。

## 現状

- **平文保存**: `api_key` / `client_secret` / `refresh_token` は `config.toml` へ
  平文で保存される。暗号化は行われていない（ADR-0098）。
- **露出緩和は維持**: `REDACTED_SECRET`（`core/src/config.rs:13`）、
  許可リスト型 DTO の `has_api_key` / `has_client_secret` / `has_refresh_token`、
  および `OpenAiConfig` / `GoogleDriveConfig` の手実装 `Debug` マスクが有効。
  IPC レスポンスに秘密値は現れない（ADR-0098 / 0102 / 0186）。
- **ファイル権限**: `AppConfig::save()` は Unix で 0600（新規作成・既存上書きの
  両方、tmp + rename、ADR-0209）。非 Unix fallback はパーミッション指定が無く、
  ACL は設定されない。owner-only の保証は Unix 系のみ
  （`core/src/config.rs:541-567`）。
- **IPC 書き込み**: `update_ai_config` の `api_key` のみ明示的な新値を書き込み可
  （ADR-0186 の受容済みリスク）。ほかの秘密フィールドは DTO 境界で正規化され、
  `config.toml` が権威。
- **Wasm**: config 永続化自体が無い（ADR-0205 で `WasmConfig` を削除、
  ADR-0193 で browser の設定フォームは読み取り専用）。

## 決定

- **設計受容を採用する**。現状の平文保存 + redaction + Unix 0600 +
  `api_key` のみ IPC 書き込み可を受容済みリスクとして明記する。
- OS キーチェーン移行は ADR-0098 の「未決定」へ集約し、本 ADR を追跡先として
  明示する。今回は実装しない。
- コード / `Cargo.toml` / `Cargo.lock` / E2E は変更せず、新規依存も追加しない。
  キーチェーン実装は将来の独立した判断として残す。
- 個別 TODO は新設せず、残リスクの追跡は ADR-0098 / 本 ADR の「未決定」に一本化
  する（ADR-0224 の規約に従う）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| keyring（新規直接依存・target feature） | 却下: 新規直接依存の選定と target / feature の一次確認が必要。Wasm 対象外で native のみ |
| secret-service（dbus/zbus 再利用だが新規直接依存） | 却下: 既存の推移依存を再利用できるが新規直接依存が生じ、Linux desktop に限定される |
| 独自 file（平文残留） | 却下: 復号鍵の置き所が残り、平文保存を実質的に解消できない |
| 設計受容（採用） | 機能を変えず露出面の緩和を維持し、判断に必要な未決定を ADR へ集約できる |

## 根拠

- ADR-0098: `skip_serializing` は再起動後の復元を壊し機能回帰するため保存形式を維持。
- ADR-0102: `extra_headers` も DTO と `Debug` からマスク済みで、IPC の露出面は
  DTO 境界で封じられている。
- ADR-0186: 生 config を受け取るコマンドも DTO 境界へ正規化し、
  `api_key` の受け入れだけを受容済みリスクとして明示している。
- ADR-0224: 残リスクは個別 TODO を作らず、ADR の「未決定」を唯一の追跡先とする。
- Wasm では config 永続化自体が無いため、キーを預かる先が存在しない。
- CI は秘密情報を使用せず、秘密フィールドは env マップの対象外
  （ADR-0204:33-34）。そのため env 注入ができず、実キーストアの自動検証は設計上不能。
- owner-only は Unix の 0600 のみで、非 Unix fallback には ACL 設定が無い。

## 未決定

- 移行順序と失敗時挙動: 既存 `config.toml` の値をいつキーチェーンへ写すか、
  書き込み失敗時にどちらを権威とするか。
- ローテーション: キーの更新・失効・アカウント切替の手順。
- プラットフォーム差: Linux secret-service / macOS Keychain /
  Windows Credential Manager の対応範囲と挙動差。
- Wasm / localStorage の平文性: 読書位置・設定・エディタ下書きは平文のまま。
- CI 実キーストア検証: secret 不使用・env 注入不可のため検証手段が未定義。
- keyring クレートの target / feature 一次確認: 未実施（選定前に必要）。

## 注記: コメントと実装の矛盾

- `core/src/config.rs:265` の `/// OAuth client secret (encrypted)` と
  `:267` の `/// Refresh token (encrypted)` は実装と矛盾する。
  実際には暗号化されず平文保存され、Unix 0600 のファイル権限のみが適用される。
- 本 ADR は事実を開示するもので、コードの doc コメントは変更しない
  （GOAL の制約: コードは変更しない）。

## 影響

- 秘密情報の保存形式・IPC 契約・ファイル権限・E2E 件数は不変（E2E 255 維持）。
- 新規依存は無く、CI / ローカルゲートの構成も変わらない。
- キーチェーン移行を検討するときは、本 ADR と ADR-0098 の未決定を満たす
  設計案と検証手段を先に定義する必要がある。

## 検証

- 文書のみの変更。コード / `Cargo.toml` / `Cargo.lock` / E2E は未変更。
- `npx markdownlint-cli2 'docs/**/*.md' README.md todo.md`: 0 issues。

## 参照

- [ADR-0098: 秘密情報の露出緩和](0098-secret-exposure-mitigation.md)
- [ADR-0102: extra_headers の IPC マスク](0102-extra-headers-redaction.md)
- [ADR-0186: 設定 IPC 境界の強化](0186-config-ipc-boundary.md)
- [ADR-0204: 環境変数マッピングの明示化](0204-env-mapping.md)
- [ADR-0205: WasmConfig の削除](0205-wasm-config-removal.md)
- [ADR-0209: 設定保存の原子化](0209-atomic-config-save.md)
- [ADR-0224: 古い TODO の ADR 実績による整理](0224-todo-reconciliation.md)
