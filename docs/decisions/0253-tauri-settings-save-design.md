---
type: Decision
title: "ADR-0253: Tauri 設定保存 UI の段階設計"
description: "browser の読み取り専用 fail-closed を維持し、desktop のみ最小 IPC と実装前提を順に満たして保存 UI を接続する段階設計を記録する。"
tags:
  - ADR
  - desktop
  - config
  - ui
generated:
  by: dsh
  at: "2026-11-20T00:14:56+09:00"
status: accepted
---

## 背景

R6 A05 残として、Tauri 設定保存 UI の接続を設計する（swarm-coding / design）。
R1 監査で次が判明した。

- `desktop/tauri.conf.json` に `withGlobalTauri` が無く、
  ブラウザ版 WebView から `window.__TAURI.core.invoke` を取得できない。
  Tauri 公式はグローバル経路に `withGlobalTauri: true` を求めるため、
  **desktop も `Err("desktop only")` になる可能性がある**（実機未検証）。
  `app.security.csp` の `script-src` は `'unsafe-inline'` を含む。
  3 経路の比較と設計受容は [ADR-0264: Tauri IPC 経路の設計受容](0264-tauri-ipc-path-acceptance.md) に記録した。
- `ConfigView.revision: u64` は `get_config` が返すが、
  `update_config_patch(state, patch: ConfigPatch)` の `expected` は
  `state.config_with_revision()` が内部で取得し、UI 側の revision を
  DTO で受け取らない（`desktop/src/commands/config.rs:27-37,606-621`）。
- 更新コマンドは `Result<T, String>` で、`map_err(e.to_string())` が
  `CoreError::Conflict` を含む生エラーを文字列へ平坦化する。
  このため UI は CAS 衝突と他の失敗をコードで判別できない。
- 設定フォームの a11y 契約（label と入力の対応、エラー表示、
  操作結果の告知）が文書化されていない。
- AI / Lint / format の 4 操作は ADR-0232 の初版外で、今回の対象に含めない。

本次は設計のみであり、コード・CSP・capability・E2E は変更しない。

## 決定

- browser 単体は読み取り専用 / fail-closed を維持する。
  ブラウザ版の設定画面は表示のみで、変更・保存を誤って約束しない。
- desktop のみ、後述の実装前提を順に満たしたうえで段階的に保存 UI を接続する。
  即時の全面接続は行わず、各前提の検証が済んだ段階ごとに進める。
- AI / Lint / format の 4 操作は初版外とし、本設計の対象外に保つ
  （ADR-0232 を継続）。

### 最小 IPC

設定画面から呼ぶコマンドは次に限定する。

| コマンド | 役割 | 備考 |
| --- | --- | --- |
| `get_config` | `ConfigView`（マスク済み DTO + `revision`）の取得 | 読み取り専用 |
| `update_config_patch` | 公開フィールドの部分更新と最新 `ConfigView` の返却 | ADR-0131 の既存コマンドを土台とする |

- `update_config_patch` は更新後の `ConfigView` を返すため、
  成功時に再 fetch する必要はない（ADR-0131）。
  ただし **reset / clear は「再 fetch が不要」という理由でこの最小 IPC に含めない**。
  `reset_config` / `clear_secrets` は意味と権限が異なる別操作であり、
  保存 UI 接続とは分離して個別に判断する（ADR-0134 / 0135）。
- fs / ai / window / updater に属するコマンドは、設定画面から呼ばない。
  設定 IPC の DTO 境界を通さないこれら操作を UI の入力へ結びつけない。

### 実装前提（順序どおりに進める）

1. `withGlobalTauri` と CSP の方針を確定する。
   単独有効化は攻撃面を広げるため、CSP（`script-src` の `'unsafe-inline'` と
   inline script の扱い）と同時に設計する（ADR-0128 / 0130）。
   経路の比較と未決定は [ADR-0264: Tauri IPC 経路の設計受容](0264-tauri-ipc-path-acceptance.md) を参照し、決定後に本設計の実装へ進む。
2. capability を最小権限で設計する。
   Tauri 2.12.5 では `invoke_handler` に登録した独自コマンドは ACL 非対象
   （R1 監査で却下済みの「capability 不在で全拒否」を維持しない）。
   ここでの最小権限は、プラグイン fs / dialog / shell の scope を
   設定画面へ与えないことと、将来的に追加する plugin permission を
   必要最小限に抑えることを指す。
3. IPC 契約を強化する。
   `ConfigUpdateRequest.expected` で UI が取得した revision を明示的に渡し、
   構造化エラー `{ code, message }` で CAS 衝突を判別可能にし、
   更新後は最新の `ConfigView` を返す（現状は内部取得の `expected` と
   `Result<T, String>` のため判別不能）。
4. a11y 契約を維持する。
   `label` の `for` 対応、`aria-invalid`、`aria-describedby`、
   操作結果を告知する `role="status"` を保存 UI でも満たす。
   ADR-0226 の固定文言と redaction 契約を継続する。
5. E2E を整備する。
   browser fallback の既存チェックを維持したうえで、
   Tauri mock の成功経路を新規 check +1 で追加する
   （期待総数はコード出典から取得し、文書側で手計算しない）。
6. 実 Tauri WebView で検証する。
   mock ではなく実際の WebView で IPC 往路・復路と表示を確認する。

### 命名の正規化（2026-11-20 追記）

- 本文の `ConfigUpdateRequest.expected` は設計時の仮名であり、実装では
  `ConfigPatch.expected` が正（ADR-0269 / 0270）。`ConfigPatch` に必須の
  `expected` を追加する形で決着し、別型の `ConfigUpdateRequest` は導入しない。
- 本文の `window.__TAURI.core.invoke` も設計時の仮名であり、実装では
  `window.__TAURI__.core.invoke`（末尾 `__` 付き）が正（ADR-0270）。
- 旧名は履歴として保持する。実装・新規文書では上記の正しい名前を使い、
  DTO 往復の残作業は [ADR-0271](0271-settings-ui-scope.md) へ集約する。

### supersede の範囲

- ADR-0128（設定フォームの読み取り表示と保存見送り）および
  ADR-0193（設定フォームの読み取り専用化）について、
  **desktop 側の保存見送り / 読み取り専用のみを本 ADR で supersede する**。
- browser 側の読み取り専用 / fail-closed 契約は supersede せず、維持する
  （ADR-0193 の常時説明、`has_api_key` の存在フラグ表示を含む）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 即保存実装 | 採らない。`withGlobalTauri` / CSP / capability / CAS / a11y の前提が未確定で、fail-closed と秘密保護を壊すリスクが高い |
| 段階設計（採用） | 読み取り専用を維持しつつ前提を順序立てて解消でき、各段階で検証できる |
| 完全見送り | 採らない。desktop の保存要求を放置し、ADR-0128 / 0193 の未決定が再浮上する |

根拠とする ADR:

- ADR-0102: `extra_headers` を IPC DTO と Debug からマスクする境界。
- ADR-0128: 読み取り表示と保存見送り。保存は withGlobalTauri / CSP 強化後に接続する前提を確立。
- ADR-0131: `ConfigPatch` と `update_config_patch`。更新後の `ConfigView` 返却で再 fetch 不要。
- ADR-0186: 生 config でも DTO 境界へ正規化し、機微フィールド注入を防ぐ契約。
- ADR-0193: ブラウザ版の読み取り専用化。誤解を与えない UI 契約を定義。
- ADR-0211: 3 設定レイヤーの権威・寿命分離。IPC DTO を単一の真実へ統合しない。
- ADR-0226: status 文言の日本語統一と redaction。固定文言のみを告知する。
- ADR-0232: 初版スコープ。AI / Lint / format を初版外と切り分ける。
- ADR-0250: 重複型を削除し経路を一本化する前例。DTO の責任分界を明確にする。

## 影響

- 本次は設計のみ。コード・CSP・capability・E2E・GOAL.md は変更しない。
- desktop の保存 UI は、実装前提 1〜6 が満たされるまで接続されない。
- browser 単体の fallback 表示と fail-closed は従来どおり維持する。
- E2E 255 件は不変（新規 check +1 は実装時に追加する）。
- 秘密情報・署名鍵・外部サービスは扱わない。

## 未決定

- 設定保存 IPC の UI 起点 CAS / 構造化エラーは [ADR-0269](0269-ipc-cas-contract.md) で実装済み。

- desktop も IPC 未接続の可能性（withGlobalTauri 未設定、実機未検証）。
  経路選択と往路検証の未決定は [ADR-0264: Tauri IPC 経路の設計受容](0264-tauri-ipc-path-acceptance.md) に集約する。
- `withGlobalTauri` の inline script と CSP nonce / hash の具体策。
- capability の実ビルド挙動（最小権限の記述が実際の ACL にどう反映されるか）。
- ~~`ConfigUpdateRequest.expected` の DTO 形（既存 `ConfigPatch` との統合または別型）~~
  は ADR-0269 で `ConfigPatch.expected` として解決済み。完全な web 側 DTO の形は
  [ADR-0271](0271-settings-ui-scope.md) の未決定へ集約。
- conflict 時の UI（再取得して再試行するか、固定文言のみ告知するか）。
- `api_key` 非対称（存在フラグのみ保持し、更新・削除ができない契約）の扱い。
- `local_roots`（新規 id の空 path を許容するか、ADR-0131 の未決定を継続）。
- 実機検証（実 Tauri WebView での往路・復路と a11y の目視確認）。

## 検証

- 文書形式: OKF frontmatter、`##` からの見出し階段、RFC 3339 `+09:00`。
- `npx markdownlint-cli2 ...`: 0 issues。
- コード・E2E は未変更。R6 で `./scripts/verify.sh` を再実行し、
  全ゲート成功 / E2E **255 passed, 0 failed** を確認した。
- `npx markdownlint-cli2 'docs/**/*.md' README.md todo.md`: 0 issues。
