---
type: Decision
title: "ADR-0279: updater チェック呼び出しの検出パターン拡張"
description: "静的ゲートが Tauri v2 の実呼び出し `.check()` を見逃す fail-open を修正する決定を記録する。"
tags:
  - ADR
  - ci
  - updater
generated:
  by: dsh
  at: "2026-11-20T23:51:00+09:00"
status: accepted
---

## 背景

R1 監査で、二つの静的ゲートが
`UpdaterExt::check` と `.check_update(` などしか検出せず、
Tauri v2 の実際の呼び出しである `updater.check().await` と
`app.updater()?.check().await` を検出しないことが分かった。
このままだと、更新チェックを配線してもゲートが成功する。

## 決定

- `scripts/check-updater-honesty.sh` と
  `scripts/check-capability-scope.sh` に
  `\.check\s*\(` を追加する。
- `scripts/check-capability-scope.sh` には
  `\.check_updates?\s*\(` も追加し、
  `updater-honesty.sh` との検出対象をそろえる。
- 直後に `(` を要求することで、
  設定フィールド名 `check_updates` や
  `.checked()` との誤爆を避ける。
- 既存の禁止パターンは維持する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| `.check(` を追加（採用） | Tauri v2 の実呼び出しを検出できる |
| 現状維持 | 更新チェック配線がゲートをすり抜ける |
| capability を削除 | 設定画面の IPC 限定ゲートを失う |

## 影響

- 現行ツリーには `.check(` が無いため、通常 run は成功する。
- コメント内や無関係なレシーバの `.check(` も検出対象になる。
  現行ツリーでは誤爆していないが、今後の誤爆リスクとして残す。
- E2E check 名・件数（256）は変更しない。

## 検証（Runtime Proof）

- `bash scripts/check-updater-honesty.sh`: rc=0。
- `bash scripts/check-capability-scope.sh`: rc=0。
- 一時コピーへ `updater.check().await` を追加:
  `check-updater-honesty.sh` は rc=1。
- 一時コピーへ `updater.check().await` を追加:
  `check-capability-scope.sh` は rc=1。
- 一時コピーへ `foo.check_updates()` を追加:
  `check-capability-scope.sh` は rc=1。
- 通常 run: `check_updates` フィールド参照は誤爆せず rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**。

## 未決定

- `download()` / `install()` / `download_and_install()` などの
  他 updater API の検出。
- UFCS 形式 `<T as UpdaterExt>::check` の検出。
- 走査対象を `desktop/src/main.rs` と `web/src/lib.rs` 以外へ広げるか。
- コメント・文字列をマスクして誤爆を減らすか。
- pubkey / endpoint / 署名を導入する際のゲート拡張条件。
