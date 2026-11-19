---
type: Decision
title: "ADR-0278: 設定画面 IPC の静的スコープゲート"
description: "設定画面から呼ぶ Tauri command を get_config / update_config_patch に限定する静的ゲートを記録する。"
tags:
  - ADR
  - tauri
  - security
generated:
  by: dsh
  at: "2026-11-20T07:43:00+09:00"
status: accepted
---

## 背景

ADR-0253 / 0274 は A05 前提 2「capability 最小権限」を列挙したが、
完了証跡が無かった。Tauri 2.12.5 では `invoke_handler` に登録した
独自 command は ACL 非対象（ADR-0253:73-74）で、
plugin fs / dialog / shell の scope が広い点は残リスクとして残る。
現状の設定画面は `get_config` / `update_config_patch` のみを呼ぶ。

## 決定

- `scripts/check-capability-scope.sh` を追加する。
- web 側の `invoke(...)` / `invoke_update(...)` 第 1 引数リテラルを抽出し、
  許可集合 `{get_config, update_config_patch}` との完全一致を検査する。
- 欠落 / 過剰、`plugin:` 参照、`UpdaterExt::check` / `checkUpdates` /
  `check_update` を検出したら fail-closed で rc=1 とする。
- `desktop/src/main.rs` の `generate_handler!` から 2 command の登録を確認し、
  抽出不能時も rc=1 とする。
- `verify.sh` と `ci.yml` の `extra_gates` から同じ script を呼ぶ。
- capability ファイルは追加しない（独自 command は ACL 非対象）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| web 側の静的スコープゲート（採用） | 設定画面の IPC 表面を回帰から守れる |
| capability ファイル追加 | 独自 command が ACL 非対象のため効果が限定的 |
| 何もしない | A05 前提 2 の完了証跡が残らない |

## 影響

- このゲートは **web 側の静的解析**であり、
  runtime の権限制限ではない。
- 危険 command は desktop の `invoke_handler` に残り得るが、
  設定画面のコードから呼ばれないことを検査する。
- `tauri.conf.json` の plugin scope（`$HOME/*` 等）は現状維持。
- E2E check 名・件数（256）は不変。

## 検証（Runtime Proof）

- `bash -n` / `shellcheck` / `shfmt -d`: すべて rc=0。
- 負向 test **11/11 PASS**: baseline rc=0、
  危険 command / `plugin:fs` / `UpdaterExt::check` /
  許可 command 欠落（2 種）/ handler 登録欠落 /
  `generate_handler!` 抽出不能 / web ファイル欠落 / 存在しない root は
  すべて rc=1。
- `./scripts/verify.sh`: 新ゲート rc=0、全ゲート成功、
  E2E **256 passed / 0 failed**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- plugin fs / dialog / shell の scope 見直し（`$HOME/*` 等は広い）。
- 実 Tauri WebView での権限挙動（ADR-0273 の環境ブロッカー）。
- custom endpoint / 署名導入時の IPC 追加と許可集合の更新（ADR-0101 / 0218）。
- 他 web ファイルへ分割した場合の AST 解析範囲拡張。
