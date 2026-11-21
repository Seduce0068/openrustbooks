---
type: Decision
title: "ADR-0294: cargo audit advisory DB の鮮度担保"
description: "verify.sh の cargo audit が古い advisory DB で脆弱性を見逃さないよう、DB 鮮度と監査対象の一致を fail-closed で検査する決定。"
tags:
  - ADR
  - CI
  - audit
generated:
  by: dsh
  at: "2026-11-22T05:00:00+09:00"
status: accepted
---

## 背景

verify.sh は `cargo audit --no-fetch` を使い、ローカルキャッシュの
advisory DB に依存していた。`--stale` を指定しないため DB が古くても
脆弱性を見逃せる。また鮮度検査と実際の監査が別 DB を指す可能性もあった。

## 決定

- advisory DB の最終コミット時刻を検査し、既定 7 日以内のみ成功とする。
- `AUDIT_MAX_AGE_DAYS` で上書きできるが、正の整数かつ 90 日以下のみ許容し、
  空文字 / 0 / 非数値 / 91 以上は fail-closed で拒否する。
- DB 欠落 / `.git` 欠落 / git log 失敗・空・非数値 / 未来タイムスタンプも失敗。
- 解決した DB パスを `cargo audit --no-fetch --db "$AUDIT_DB_PATH"` へ渡し、
  鮮度検査と監査が同一 DB を指すことを保証する。
- verify.sh は `audit.toml` の `database.path` を無視し、
  `AUDIT_DB_DIR` / `CARGO_HOME` / `~/.cargo` 由来の DB を使う。
- 鮮度不合格時は `cargo audit` を実行せず E2E 前に停止する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 鮮度ゲート + 同一 DB 指定（採用） | オフラインでも古い DB を検出できる |
| `cargo audit` で fetch を有効化 | CI は可能だがローカル verify がネットワーク依存になる |
| 鮮度チェックなし | 脆弱性の見逃しを許すため却下 |

## 検証（Runtime Proof）

- 負向 test 14 件すべて PASS:
  - fresh / 7日境界内側 / 90日境界は rc=0。
  - 古い DB / 7日境界外側 / 91 / 36501 / 0 / abc / 空文字 /
    欠落 / `.git` 欠落 / 未来タイムスタンプは rc=1。
- `bash -n` / `shellcheck`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、
  audit-db-freshness rc=0、audit rc=0、
  E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- CI 側で advisory DB を定期更新する schedule workflow。
- `audit.toml` を導入する場合の DB 解決ルール統合。
